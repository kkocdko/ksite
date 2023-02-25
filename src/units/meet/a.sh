#!/bin/sh
~/misc/apps/miniserve --header Cache-Control:no-store -p 9453 $(dirname $0)
