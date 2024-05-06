#!/usr/bin/env bash
# Original Source: https://github.com/Jorl17/open-elevation/blob/master/download-srtm-data.sh

set -eu

curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_NE_250m_TIF.rar > ne.rar
unrar x ne.rar
rm ne.rar
curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_SE_250m_TIF.rar > se.rar
unrar x se.rar
rm se.rar
curl https://srtm.csi.cgiar.org/wp-content/uploads/files/250m/SRTM_W_250m_TIF.rar > w.rar
unrar x w.rar
rm w.rar
