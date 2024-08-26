#!/bin/bash

LOGFILE=/memvectordb/output.log

mkdir -p /memvectordb

if [ "$RESTORE_DB" = "true" ]; then
    echo "Starting memvectordb with database restoration..."
    /usr/local/bin/memvectordb --restore-db >> $LOGFILE 2>&1
else
    echo "Starting memvectordb"
    /usr/local/bin/memvectordb >> $LOGFILE 2>&1
fi
