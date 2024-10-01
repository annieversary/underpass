set dotenv-required

set dotenv-load := true


# build and run in development mode
run: build
    cargo run

# build and run in release mode
run-release: release
    cargo run --release

# build in develoment mode
build:
    npm run build
    cargo build

# build in release mode
release:
    npm run prod
    cp frontend/index.html public
    cargo build --release

# run tests and typecheck typescript
test:
    npm run typecheck
    npm run test
    cargo test

# get required data
get-data: data-taginfo data-elevation

# download elevation data to $DATA_PATH/elevation/
data-elevation:
    data/elevation/create-dataset.sh "$DATA_PATH/elevation"

# update $DATA_PATH/taginfo/taginfo.json
data-taginfo:
    cargo r --bin update-taginfo --release
