#!/usr/bin/env bash
# Original Source: https://github.com/Jorl17/open-elevation/blob/master/download-srtm-data.sh

set -eu

if [ -n "$1" ]; then
    OUTPUTDIR="$1"
else
    OUTPUTDIR="."
fi

curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_NE_250m_TIF.rar > $OUTPUTDIR/ne.rar
unrar x $OUTPUTDIR/ne.rar
rm $OUTPUTDIR/ne.rar
curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_SE_250m_TIF.rar > $OUTPUTDIR/se.rar
unrar x $OUTPUTDIR/se.rar
rm $OUTPUTDIR/se.rar
curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_W_250m_TIF.rar > $OUTPUTDIR/w.rar
unrar x $OUTPUTDIR/w.rar
rm $OUTPUTDIR/w.rar
