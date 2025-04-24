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

flatpakArr=()	# all installed flatpak in the format: appid/arch/branch
flatpakRawList=$(flatpak list --columns=app,arch,branch)
IFS=$'\n'
while read -r line; do
	rowArr+=("$line")
done <<< "${flatpakRawList}"
for row in ${rowArr[@]}; do
	IFS=$'\t' read -ra appArr <<< ${row}
	flatpakArr+=("${appArr[0]}/${appArr[1]}/${appArr[2]}")
done


case "$1" in
	-Q)
		if [ -z "${program}" ]; then
			for app in ${flatpakArr[@]}; do
				appVersion=$(flatpak info ${app} | grep Version | awk -F': ' '{print $2}')
				appBranch=$(flatpak info ${app} | grep Version | awk -F': ' '{print $2}')
				appID="$(echo ${app} | awk -F'/' '{print $1}')"
				appArch=$(echo ${app} | awk -F'/' '{print $2}')
				appBranch=$(echo ${app} | awk -F'/' '{print $3}')
				appName=$(flatpak list --columns=name,app,arch,branch,app | grep "${appID}	" | grep "${appArch}	" | grep "${appBranch}	" | awk -F'\t' '{print $1}')
				[ -z "$appVersion" ] && appVersion="?"
				echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
			done
		else
			preapp=$(flatpak --columns=app,branch search ${program} | awk -F'\n' '{print $1}')
			IFS=$'\t' read -ra appArr <<< ${preapp}
			app="${appArr[0]}//${appArr[1]}"
			appVersion=$(flatpak info ${app} | grep Version | awk -F': ' '{print $2}')
			appBranch=$(flatpak info ${app} | grep Version | awk -F': ' '{print $2}')
			appID="$(echo ${app} | awk -F'/' '{print $1}')"
			appArch=$(echo ${app} | awk -F'/' '{print $2}')
			appBranch=$(echo ${app} | awk -F'/' '{print $3}')
			appName=$(flatpak list --columns=name,app,arch,branch,app | grep "${appID}	" | grep "${appArch}	" | grep "${appBranch}	" | awk -F'\t' '{print $1}')
			[ -z "$appVersion" ] && appVersion="?"
			echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
		fi
		;;
	-Q*) ;;
	-S*) ;;
	-R*)	;;
	-F*)	;;
esac
#flatpak info 

#fullList=$(flatpak list --app --columns=application,version,name | awk '{print "\033[1m"$3"\033[0m", "(" $1 ")", "\033[32m"$2"\033[0m"}')
