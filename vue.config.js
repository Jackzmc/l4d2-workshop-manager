const { writeFileSync } = require('fs')
let pkg = require('./package.json')

process.env.VUE_APP_VERSION = pkg.version
pkg.build = (parseInt(pkg.build || 0)) + 1
writeFileSync('./package.json', JSON.stringify(pkg, null, 2), 'utf-8')

process.env.VUE_APP_BUILD = pkg.build