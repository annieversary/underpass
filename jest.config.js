module.exports = {
    transform: {
        "\\.[jt]sx?$": ['esbuild-jest', {
            loaders: {
                '.spec.js': 'jsx',
                '.js': 'jsx'
            }
        }
        ]
    },
    testPathIgnorePatterns: ['/node_modules/', '/public/'],
    moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node']
};
