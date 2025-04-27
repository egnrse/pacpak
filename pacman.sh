#!/bin/env bash
# shallow flatpak integration into pacman (proof of concept)
# (by egnrse)

## === SETTINGS ===
SPEED=""	#if not empty: skip fields that would take some more time
skippedMsg="[skipped]"	# message for files skipped because for $SPEED
notImplemented="[not implemented]"
appTODO="[todo]"
appNOTLOCAL="[not found locally]"


## === CONSTANTS ===
normal="\e[0m"
bold="\e[1m"
green="\e[32m"

args=$@
program=$2	# file or program given

## === PACMAN ===
# execute the command normally
#pacman ${args}	#dev
# TODO wrap output/error

## === FUCNTIONS ===
# convert a given date format to the format that pacman normally uses and return it
convertDate() {
	local dateIn=$1
	#echo "${dateIn}"	# only for debuging
	echo $(date --date="${dateIn}" +"%a %d %b %Y %I:%M:%S %p %Z")
}

# resets/fills flatpakArr
# arg1: extra argument to the flatpak command (without the dashes) (eg. apps)
initArr() {
	arg1=${1:""}
	if [ -n "${arg1}" ]; then
		arg1="--$arg1"
	fi
	# flatpakArr is an array of all installed flatpaks with the format: appid/arch/branch (called extended appID in this program)
	flatpakArr=()
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

# returns an extended appID (or more) if there are matches with $1 in the flatpakArr
searchAppID() {
	echo $1 #dev
	local searchL=$1
	# detect empty input and grep in this case for everything
	if [ -z "$searchL" ]; then searchL="."; fi
	if [ -z "flatpakArr[@]" ]; then echo "WARNING (searchAppID): $flatpakArr is empty, this function assumes flatpakArr to be filled"; fi;
	for row in ${flatpakArr[@]}; do
		echo ${row} | grep -i $searchL
	done
}

# get (short) info about the app given at $1 (saved to app* variables)
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
	([ -z "$appVersion" ] || [[ "$appVersion" == "$appID" ]] ) && appVersion="?"
}
# get all fields but slower than getAppInfo()
getAppInfoFull() {
	local appL=$1
	appInfo=$(flatpak info ${appL})
	appName=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if /.(.*?) - /s')
	appDescription=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if / - (.*?)ID:/s')
	appDescription=$(awk 'BEGIN{ORS=""} {print $0}' <<< "$appDescription")	# strip trailing newlines

	appID=$(echo "${appInfo}" | grep ID: | awk -F': ' '{print $2}')
	appRef=$(echo "${appInfo}" | grep Ref: | awk -F': ' '{print $2}')
	appArch=$(echo "${appInfo}" | grep Arch: | awk -F': ' '{print $2}')
	appBranch=$(echo "${appInfo}" | grep Branch: | awk -F': ' '{print $2}')
	appVersion=$(echo "${appInfo}" | grep Version: | awk -F': ' '{print $2}')
	[ -z "$appVersion" ] && appVersion="?"	# deal with empty version fields
	appLicense=$(echo "${appInfo}" | grep License: | awk -F': ' '{print $2}')
	appOrigin=$(echo "${appInfo}" | grep Origin: | awk -F': ' '{print $2}')
	appCollection=$(echo "${appInfo}" | grep Collection: | awk -F': ' '{print $2}')
	#Installation?
	appInstallSize=$(echo "${appInfo}" | grep Installed: | awk -F': ' '{print $2}')
	appRuntime=$(echo "${appInfo}" | grep Runtime: | awk -F': ' '{print $2}')
	appBuildDate=$(echo "${appInfo}" | grep Date: | awk -F': ' '{print $2}')
	appBuildDate=$(convertDate ${appBuildDate})

	## Custom Fields ##
	if [[ "$appOrigin" == "flathub" ]]; then
		#TODO get source url if availabe (on flathub)?
		appUrl="https://flathub.org/apps/${appID}"
	else
		appUrl="Installed from '${appOrigin}'"
	fi
	if [ ${SPEED} ]; then
		# try here, before giving up
		local globalFlatpakPath="/var/lib/flatpak/app"
		if [ $(ls ${globalFlatpakPath}) ]; then
			local preDate="$(stat ${globalFlatpakPath}/${appL}) | grep Modify | awk -F': ' '{print $2}')"
			appInstallDate=$(convertDate ${preDate})
		else
			appInstallDate="${skippedMsg}"
		fi
	else
		local preDate="$(stat $(flatpak info --show-location ${appL}) | grep Modify | awk -F': ' '{print $2}')"
		appInstallDate=$(convertDate ${preDate})
	fi
}

# TODO: print the Qi data for application $1
printAppInfo() {
	local appL=$1
	local appNONE="None"
	
	# get values for $appL
	getAppInfoFull $appL

	# name
	local appNameL=""
	if [ -n "$appName" ]; then appNameL="($appName)"; fi
	echo -e "${bold}Name		:${normal} $appID $appNameL"
	echo -e "${bold}Version		:${normal} $appVersion"
	echo -e "${bold}Description	:${normal} $appDescription"
	echo -e "${bold}Architecture	:${normal} $appArch"
	echo -e "${bold}URL		:${normal} $appUrl"
	echo -e "${bold}Licenses	:${normal} $appLicense"

	if [ -z "${appCollection}" ]; then appCollection="$appNONE"; fi
	echo -e "${bold}Groups		:${normal} $appCollection"	#change?
	
	# TODO: how to get this?
	echo -e "${bold}Provides	:${normal} $appTODO"

	if [ -z "${appRuntime}" ]; then appRuntime="$appNONE"; fi
	echo -e "${bold}Depends On	:${normal} $appRuntime"

	echo -e "${bold}Optional Deps	:${normal} $notImplemented"
	
	# if it needs a runtime it is an app?
	if [ -n "${appRuntime}" ]; then appRequired="$appNONE"; fi
	if [ -z "${appRequired}" ]; then appRequired=$appNONE; fi;
	echo -e "${bold}Required By	:${normal} $appRequired"

	echo -e "${bold}Optional For	:${normal} $notImplemented"
	echo -e "${bold}Conflicts With	:${normal} $notImplemented"
	echo -e "${bold}Replaces	:${normal} $notImplemented"
	echo -e "${bold}Installed Size	:${normal} $appInstallSize"
	# how to get packager? (flathub directly?/using ostree)
	echo -e "${bold}Packager	:${normal} $appNOTLOCAL"
	echo -e "${bold}Build Date	:${normal} $appBuildDate"	# change format?
	echo -e "${bold}Install Date	:${normal} $appInstallDate"

	if [ -z "" ]; then
		echo -e "${bold}Install Reason	:${normal} $appTODO"
	else
		# check if it is under flatpak list --app
		echo -e "${bold}Install Reason	:${normal} Explicitly installed"
		echo -e "${bold}Install Reason	:${normal} Installed as a dependency for another package"
	fi
	echo -e "${bold}Install Script	:${normal} $notImplemented"
	echo -e "${bold}Validated By	:${normal} $notImplemented"
}

## === START ===

# search for an extended appID (and conert program to this format)
#program=$(searchAppID $program)

flatpakArr=()	# all installed flatpaks in the format: appid/arch/branch (extended appID)
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
	-S*)
		#dev
		#searchAppID ${program}
		;;
	-R*)	;;
	-F*)	;;
esac
