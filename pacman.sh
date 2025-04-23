#!/bin/env bash
# shallow flatpak integration into pacman (proof of concept)
# (by egnrse)

normal="\e[0m"
bold="\e[1m"
green="\e[32m"

args=$@

# execute the command normally
#pacman ${args}	#dev
# TODO wrap output/error

program=$2	# file or program
# TODO search if program is not a exact match (for some args)

# does not work because of different branches (that might be installed at the same time)
flatpakList=$(flatpak list --columns=application)
echo $flatpakList


case "$1" in
	-Q)
		if [ -z "${program}" ]; then
			for appid in ${flatpakList}; do
				# TODO deal with branches
				flatpakBr=$(flatpak list --columns=application,branch | grep ${appid})
				#echo ${flatpakBr}
				#version=$(flatpak info ${appid} | grep Version | awk '{print $2}')
				#echo -e "${bold}${appid} ${green}${version}${normal}"
			done
		else
			echo ${appid}
		fi
		;;
	-Q*) ;;
	-S*) ;;
	-R*)	;;
	-F*)	;;
esac
#flatpak info 

#fullList=$(flatpak list --app --columns=application,version,name | awk '{print "\033[1m"$3"\033[0m", "(" $1 ")", "\033[32m"$2"\033[0m"}')
