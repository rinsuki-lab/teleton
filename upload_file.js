import fs from "node:fs"
import crypto from "node:crypto"

const server = process.argv[2]
const filePath = process.argv[3]

const stat = fs.statSync(filePath)

const r = await fetch(`${server}/v1/upload/start?file_size=${stat.size}`, {
    method: "POST",
}).then(r => r.json())

if (typeof r.token !== "string") {
    throw new Error(`!?`)
}

const { token } = r

const fd = fs.openSync(filePath, "r")

const CHUNK_SIZE = 512 * 1024
const uploads = []
const hasher = crypto.createHash("md5")

for (let i=0; i<stat.size; i+=CHUNK_SIZE) {
    const buffer = new Uint8Array(CHUNK_SIZE)
    const d = fs.readSync(fd, buffer, 0, Math.min(
        CHUNK_SIZE,
        stat.size - i
    ), i)
    hasher.update(buffer.slice(0, d))
    const prog = (i + d) / stat.size
    const args = [i + d, stat.size, prog]
    const res = uploads.push(fetch(`${server}/v1/upload/chunk?offset=${i}&token=${token}`, {
        method: "POST",
        body: buffer.slice(0, d),
    }).then(r => [r, args]))
} 

for (const upload of uploads) {
    const [res, prog] = await upload
    console.log(res.status, await res.text(), ...prog)
}

const finalize_res = await fetch(`${server}/v1/upload/finalize?token=${token}`, {
    method: "POST",
    headers: {
        "Content-Type": "application/json"
    },
    body: JSON.stringify({
        md5: hasher.digest("hex"),
        name: "test.bin"
    })
})

const res = await finalize_res.text()

try {
    const { ref } = JSON.parse(res)
    console.log(`${server}/v1/chunk/range/${ref}`)
} catch(e) {
    console.log(res)
}
