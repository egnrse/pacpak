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

##=== functions ===
# resets/fills flatpakArr
# arg1: extra argument to the flatpak command (without the dashes) (eg. apps)
initArr() {
	arg1=${1:""}
	if [ -n "${arg1}" ]; then
		arg1="--$arg1"
	fi
	flatpakArr=()	# all installed flatpak in the format: appid/arch/branch
	flatpakRawList=$(flatpak list ${arg1} --columns=app,arch,branch)
	IFS=$'\n'
	while read -r line; do
		rowArr+=("$line")
	done <<< "${flatpakRawList}"
	for row in ${rowArr[@]}; do
		IFS=$'\t' read -ra appArr <<< ${row}
		flatpakArr+=("${appArr[0]}/${appArr[1]}/${appArr[2]}")
	done
}
# get info about the app given at $1
getAppInfo() {
	local appL=$1
	if [ -z "${flatpakFullList}" ]; then
		flatpakFullList=$(flatpak list --columns=name,arch,branch,version,app)
	fi
	appID="$(echo "${appL}" | awk -F'/' '{print $1}')"
	appArch="$(echo "${appL}" | awk -F'/' '{print $2}')"
	appBranch="$(echo "${appL}" | awk -F'/' '{print $3}')"
	
	local appRow=$(echo "${flatpakFullList}" | grep "${appID}$" | grep "${appArch}	" | grep "${appBranch}	")
	local appArr=()
	IFS=$'\t' read -ra appArr <<< ${appRow}
	appName=${appArr[0]}
	appVersion=${appArr[3]}
	([ -z "$appVersion" ] || [[ "$appVersion" == "$appID" ]] )&& appVersion="?"
}
# TODO: get all fields but slower
getAppInfoFull() {
	local appL=$1

	
	appInfo=$(flatpak info ${appL})
	appName=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if /.(.*?) - /s')
	# TODO
	appDescription=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if / - (.*?)ID:/s')
	appID=$(echo "${appInfo}" | grep ID: | awk -F': ' '{print $2}')
	appRef=$(echo "${appInfo}" | grep Ref: | awk -F': ' '{print $2}')
	appArch=$(echo "${appInfo}" | grep Arch: | awk -F': ' '{print $2}')
	appBranch=$(echo "${appInfo}" | grep Branch: | awk -F': ' '{print $2}')
	appVersion=$(echo "${appInfo}" | grep Version: | awk -F': ' '{print $2}')
	[ -z "$appVersion" ] && appVersion="?"
	appLicense=$(echo "${appInfo}" | grep License: | awk -F': ' '{print $2}')
	#origin?
	appCollection=$(echo "${appInfo}" | grep Collection: | awk -F': ' '{print $2}')
	#Installation?
	appInstallSize=$(echo "${appInfo}" | grep Installed: | awk -F': ' '{print $2}')
	appRuntime=$(echo "${appInfo}" | grep Runtime: | awk -F': ' '{print $2}')
	appDate=$(echo "${appInfo}" | grep Date: | awk -F': ' '{print $2}')

}
# TODO: print the Qi data for a application
printAppInfo() {
	local appL=$1
	getAppInfoFull $appL
	local appNameL=""
	if [ -z "$appName" ]; then
		allNameL="($appName)"
	fi
	echo -e "${bold}Name		:${normal} $appID $appNameL"
	echo -e "${bold}Version		:${normal} $appVersion"
	echo -e "${bold}Description	:${normal} $appDescription"
	echo -e "${bold}Architecture	:${normal} $appArch"
	# from flathub?
	echo -e "${bold}URL		:${normal} $appTODO"
	echo -e "${bold}Licenses	:${normal} $appLicense"
	echo -e "${bold}Groups		:${normal} $appCollection"	#change?
	echo -e "${bold}Provides	:${normal} $appTODO"
	echo -e "${bold}Depends On	:${normal} $appRuntime"
	echo -e "${bold}Optional Deps	:${normal} $appTODO"
	# only if runtime?
	echo -e "${bold}Required By	:${normal} $appTODO"
	echo -e "${bold}Optional For	:${normal} $appTODO"
	echo -e "${bold}Conflicts With	:${normal} $appTODO"
	echo -e "${bold}Replaces	:${normal} $appTODO"
	echo -e "${bold}Installed Size	:${normal} $appInstallSize"
	# how to get packager? (flathub directly?)
	echo -e "${bold}Packager	:${normal} $appTODO"
	echo -e "${bold}Build Date	:${normal} $appDate"	# change format?
	# stat /var/lib/flatpak/app/<app-id> | grep Modify
	echo -e "${bold}Install Date	:${normal} $appTODO"
	if [ -z "" ]; then
		echo -e "${bold}Install Reason	:${normal} TODO"
	else
		# check if it is under flatpak list --app
		echo -e "${bold}Install Reason	:${normal} Explicitly installed"
		echo -e "${bold}Install Reason	:${normal} Installed as a dependency for another package"
	fi
	echo -e "${bold}Install Script	:${normal} $appTODO"
	echo -e "${bold}Validated By	:${normal} $appTODO"
}


program=$2	# file or program
# TODO search if program is not a exact match (for some args) (faster than search)

flatpakArr=()	# all installed flatpak in the format: appid/arch/branch
initArr;
flatpakFullList="" # big list of the installed flatpak


case "$1" in
	-Q)
		if [ -z "${program}" ]; then
			for app in ${flatpakArr[@]}; do
				getAppInfo ${app};
				echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
			done
		else
			preapp=$(flatpak --columns=app,branch search ${program} | awk -F'\n' '{print $1}')
			IFS=$'\t' read -ra appArr <<< ${preapp}
			app="${appArr[0]}//${appArr[1]}"
			getAppInfo ${app};
			echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
		fi
		;;
	-Q*) 
		# Qi option
		if [[ "$1" == *"i"* ]]; then
			if [ -n "$program" ]; then
				printAppInfo ${program}
			else
				echo "note: flatpaks not shown (must give a program)"
			fi
		fi
			;;
	-S*) ;;
	-R*)	;;
	-F*)	;;
esac
