#!/bin/env bash
# shallow flatpak integration into pacman (proof of concept)
# (by egnrse)

args=$@

# execute the command normally
pacman ${args}
# TODO wrap output/error

program=$2	# file or program
# TODO search if program is not a exact match (for some args)

case "$1" in
	-Q)
		fullList=$(flatpak list --app --columns=application,version,name | awk '{print "\033[1m"$3"\033[0m", "(" $1 ")", "\033[32m"$2"\033[0m"}')
		if [ -z "${program}" ]; then
			echo ${fullList}
		else
			echo ${fullList} | grep ${program}
		fi
		;;
	-Q*) ;;
	-S*) ;;
	-R*)	;;
	-F*)	;;
esac
#flatpak info 
