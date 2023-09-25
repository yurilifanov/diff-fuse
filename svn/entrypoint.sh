#!/usr/bin/env bash
svnserve -d --log-file=/svn/svn.log
svn co svn://localhost/svn/src . # running in /src
bash
