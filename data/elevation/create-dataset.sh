#!/usr/bin/env bash
# Original Source: https://github.com/Jorl17/open-elevation/blob/master/create-dataset.sh

set -eu


SCRIPTDIR="$(dirname $0)"

if [ -n "$1" ]; then
    OUTPUTDIR="$1"
else
    OUTPUTDIR="$SCRIPTDIR"
fi

mkdir -p $OUTPUTDIR

$SCRIPTDIR/download-srtm-data.sh $OUTPUTDIR
$SCRIPTDIR/create-tiles.sh $OUTPUTDIR/SRTM_NE_250m.tif 10 10
$SCRIPTDIR/create-tiles.sh $OUTPUTDIR/SRTM_SE_250m.tif 10 10
$SCRIPTDIR/create-tiles.sh $OUTPUTDIR/SRTM_W_250m.tif 10 20
rm -rf $OUTPUTDIR/SRTM_NE_250m.tif $OUTPUTDIR/SRTM_SE_250m.tif $OUTPUTDIR/SRTM_W_250m.tif
