use std::{cmp::max, time::Duration};

use grammers_client::{grammers_tl_types as tl, session::Session, Client, Config, InitParams, ReconnectionPolicy};

struct ReConPolicy;

impl ReconnectionPolicy for ReConPolicy {
    fn should_retry(&self, attempts: usize) -> std::ops::ControlFlow<(), std::time::Duration> {
        let duration = u64::pow(2, attempts as _) + 100;
        let duration = max(duration, 60 * 1000 * 1000);
        std::ops::ControlFlow::Continue(Duration::from_millis(duration))
    }
}

pub async fn get_authorized_client() -> Client {
    let session_path = std::env::var("TELETON_SESSION_PATH").expect("TELETON_SESSION_PATH is not specified");

    let api_id = std::env::var("TELETON_API_ID").expect("TELETON_API_ID is not specified").parse().expect("Failed to parse TELETON_API_ID (should be integer)");
    let api_hash = std::env::var("TELETON_API_HASH").expect("TELETON_API_HASH is not specified");
    let proxy_url = match std::env::var("TELETON_PROXY") {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    let config = Config {
        session: Session::load_file_or_create(&session_path).unwrap(),
        api_id,
        api_hash: api_hash.clone(),
        params: InitParams {
            proxy_url: proxy_url.clone(),
            reconnection_policy: &ReConPolicy,
            ..Default::default()
        },
    };

    let mut client = Client::connect(config)
        .await
        .expect("Failed to connect server");

    if !client.is_authorized().await.expect("Failed to check already authorized") {
        let req = tl::functions::auth::ExportLoginToken {
            api_id: api_id,
            api_hash: api_hash.clone(),
            except_ids: Vec::new(),
        };

        let result = client.invoke(&req).await.expect("failed to request login token");

        let result = match result {
            tl::enums::auth::LoginToken::Token(t) => t,
            _ => {
                panic!("Unexpected state {:?}", result);
            }
        };

        let url = format!("tg://login?token={}", base64::Engine::encode(&base64::prelude::BASE64_URL_SAFE, &result.token));

        let qr = qrcode::QrCode::new(url).expect("Failed to generate QR code for auth")
            .render()
            .light_color("\x1b[7m  \x1b[0m").dark_color("\x1b[49m  \x1b[0m")
            .build();

        println!("Please login with QR code:\n{}", qr);

        loop {
            // TODO: handling QR code expires (30 sec?)
            let (update, _) = client.next_raw_update().await.unwrap();
            match update {
                tl::enums::Update::LoginToken => break,
                _ => continue,
            }
        }
        let result = client.invoke(&req).await.unwrap();

        match result {
            tl::enums::auth::LoginToken::Success(s) => {},
            tl::enums::auth::LoginToken::MigrateTo(mt) => {
                // we couldn't use client.invoke_in_dc, since it requires already authenticated
                let dc_id = mt.dc_id;
                println!("Migrating to DC {}", mt.dc_id);

                let server_config = tl::functions::help::GetConfig {};
                let server_config = client.invoke(&server_config).await.expect("Failed to get server config");
                let server_config = match server_config { tl::enums::Config::Config(c) => c };

                println!("Current DC list: {:#?}", server_config.dc_options);

                let good_server = server_config.dc_options.iter().map(|x|  match x {
                    tl::enums::DcOption::Option(d) => d,
                }).find(|x| {
                    if x.id != dc_id {
                        return false;
                    }

                    if x.cdn {
                        return false;
                    }

                    if x.ipv6 {
                        // TODO: support
                        return false;
                    }

                    if x.media_only {
                        return false;
                    }

                    return true;
                }).expect("Failed to find DC");

                println!("Migrating to DC {:?}", good_server);

                let config = Config {
                    session: Session::load_file_or_create(&session_path).unwrap(),
                    api_id,
                    api_hash: api_hash.clone(),
                    params: InitParams {
                        proxy_url: proxy_url.clone(),
                        reconnection_policy: &ReConPolicy,
                        server_addr: Some(format!("{}:{}", good_server.ip_address, good_server.port).parse().expect("Failed to parse DC ip address")),
                        ..Default::default()
                    },
                };

                client = Client::connect(config).await.expect("Failed to migrating DC");

                let request = tl::functions::auth::ImportLoginToken {
                    token: mt.token,
                };
                let result = client.invoke(&request).await.unwrap();
                match result {
                    tl::enums::auth::LoginToken::Success(_) => {},
                    _ => {
                        panic!("Unknown response when migrating login token {:?}", result);
                    }
                }

                let user = tl::functions::users::GetUsers {
                    id: vec![tl::enums::InputUser::UserSelf]
                };
                let user = client.invoke(&user).await.unwrap();
                let user = &user[0];
                client.session().set_user(user.id(), mt.dc_id, false);
            },
            _ => {
                panic!("Unknown response when requesting authenticated session {:?}", result);
            }
        };

        client.session().save_to_file(&session_path).unwrap();
    }

    client
}
