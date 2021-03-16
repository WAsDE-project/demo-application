const express = require('express')
const serveIndex = require('serve-index')
const app = express()
const port = process.env.PORT || 3000

app.use(express.json())
app.use(express.static('public'))

const fs = require('fs')
const path = require('path')
const _ = require('lodash')

const get_filepaths = async () => {
    const recurse = async (filepath, filepaths) => {
        const files = await fs.promises.readdir(filepath)
        for (const file of files) {
            const fullpath = path.join(filepath, file)
            const stat = await fs.promises.stat(fullpath)
            if (stat.isFile() && file.endsWith('.json'))
                filepaths.push(fullpath)
            else if (stat.isDirectory())
                await recurse(fullpath, filepaths)
        }
        return filepaths
    }
    return await recurse('public', [])
}

/*
curl -i -X POST -H 'Content-Type: application/json' -d '{"attributes": ["aarch64", "Speaker"]}' http://localhost:3000/api/marvin@1.0.0
curl -i -X POST -H 'Content-Type: application/json' -d '{"attributes": ["aarch64", "Camera"]}' http://localhost:3000/api/marvin@1.0.0
curl -i -X POST -H 'Content-Type: application/json' -d '{"attributes": ["aarch64", "Speaker", "Camera", "Kubernetes"]}' http://localhost:3000/api/marvin@1.0.0

// Leaving body out matches any
curl -i -X POST -H 'Content-Type: application/json' http://localhost:3000/api/marvin@1.0.0
*/
app.all('/api/:id', function (req, res) {
    /*
        remove metadata not fitting with id or has attributes that our platform (request body) does not have
        From remaining modules pick the one that has most matching attributes to our platform
        In case of multiple modules pick one. In future we may consider preferring one attribute over another.
        Query: (aarch64, Mouse, Camera)
        Discard: (aarch64, Speaker)
        Accept: (aarch64, Camera)
        Accept: (aarch64, Mouse)
    */

    Promise.resolve(get_filepaths()).then(paths => {
        try {
            const fetch_metadata = (filepath) => {
                return fs.readFileSync(filepath, 'utf8')
            }
            // read and parse metadatas
            const metadatas = paths.map(fetch_metadata).map(JSON.parse)
            // filter metadata by the searched ID and discard metadata that requires attributes not listed by query
            const metadatas_filtered = metadatas.filter(m => m.id === req.params.id)
            .filter(m => !req.body.attributes || !m.attributes || m.attributes.every(a => req.body.attributes.includes(a)))

            // select best match for the attributes, which means the one that has most matching attributes
            // TODO: Weighted preference of attributes? Who decides the weights?
            const best_match = _.sortBy(metadatas_filtered, [m => m.attributes ? -m.attributes.length : 0])[0]
            if (best_match)
                return res.send(best_match)
            res.status(404).send('no match')
        } catch (err) {
            console.log(err)
            res.status(500).send(err.toString())
        }
    })
})

app.use(serveIndex('public', { 'icons': true }))

app.listen(port, () => console.log(`App listening on port ${port}!`))
