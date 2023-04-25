#!/usr/bin/env bash

set -e 

ALL="sapiens_bot sapiens_cli sapiens sapiens_derive sapiens_tools"

for a in ${ALL}; 
do
		cargo changelog -w $a
done
