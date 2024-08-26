#!/bin/bash

LOGFILE=/memvectordb/output.log

mkdir -p /memvectordb

if [ "$RESTORE_DB" = "true" ]; then
    echo "Starting memvectordb with database restoration..." | tee -a $LOGFILE
    /usr/local/bin/memvectordb --restore-db 2>&1 | tee -a $LOGFILE
else
    echo "Starting memvectordb" | tee -a $LOGFILE
    /usr/local/bin/memvectordb 2>&1 | tee -a $LOGFILE
fi
