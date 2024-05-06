#!/usr/bin/env bash
# Original Source: https://github.com/Jorl17/open-elevation/blob/master/create-dataset.sh

set -eu

# ./download-srtm-data.sh
./create-tiles.sh SRTM_NE_250m.tif 10 10
./create-tiles.sh SRTM_SE_250m.tif 10 10
./create-tiles.sh SRTM_W_250m.tif 10 20
rm -rf SRTM_NE_250m.tif SRTM_SE_250m.tif SRTM_W_250m.tif
