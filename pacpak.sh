#!/bin/env bash
# shallow flatpak integration into pacman (proof of concept)
# (by egnrse)

## === SETTINGS ===
# activate, by setting to a non empty value
WRAP="T"			# forward commands to pacman and handle its outputs (WIP)
SPEED=""		# skip fields that would take some more time
COLOR="T"			# show colors


# Qi
skippedMsg="[skipped]"	# message for files skipped because for $SPEED
notImplemented="[not implemented]"
appNOTLOCAL="[not found locally]"	# information that needs to be fetch from eg. flathub
appTODO="[todo]"


## === CONSTANTS ===
VERSION="0.0.1"
SPACER="  "			# spacer between multiple entries (for -Qi)
SPACE_VERSION="                      "	# spacer for the version display (-V)

normal="\e[0m"
bold="\e[1m"
green="\e[32m"
red="\e[31m"


args=$@
program=$2	# file or program given


# errors
err_pacNotFound="${red}${bold}error:${normal} package '${program}' was not found"


## === PACMAN ===
## functions to handle/wrap the output from pacman
pacOut=""	# stores all the output from pacman
handleOut() {
	while IFS= read -r line; do
		# ignore some errors
		if $(echo "$line" | grep "package '${program}' was not found">/dev/null); then
			:
		else
			pacOut+="$line\n"
			echo -e "$line"
		fi
	done
}
pacErr=""
handleErr() {
	while IFS= read -r line; do
		pacErr+="$line\n"
		echo -e "$line"
	done
}

if [ -n "$WRAP" ]; then
	# execute the command normally
	if [ -n "$COLOR" ]; then
		pacColor="--color always"
	fi
	pacman ${pacColor} ${args} > >(handleOut) 2> >(handleErr)
	pacReturn="$?"
else
	pacReturn="1"
fi


## === FUNCTIONS ===
# returns the date given with $1 with the format that pacman normally uses
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

# returns extended appIDs, if there are matches with $1 in the flatpakArr (returns to programArr as an array of matches)
searchAppID_local() {
	local searchL=$1
	# detect empty input and grep in this case for everything
	if [ -z "$searchL" ]; then searchL="."; fi
	if [ -z "flatpakArr[@]" ]; then echo "WARNING (searchAppID_local): \$flatpakArr is empty, this function assumes \$flatpakArr to be filled"; fi;

	programArr=()
	for row in ${flatpakArr[@]}; do
		if [ -n "$(echo ${row} | grep -i $searchL)" ]; then
			programArr+=(${row})
		fi
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
# get all availabe info fields, but slower than getAppInfo()
getAppInfoFull() {
	local appL=$1
	appInfo=$(flatpak info ${appL})
	appName=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if /.(.*?) - /s')
	appDescription=$(echo "${appInfo}" | perl -0777 -ne 'print $1 if / - (.*?)ID:/s')
	#TODO fix Description: space missing when unwrapping line
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
		appInstallDate="${skippedMsg}"
		appProvides="${skippedMsg}"
	else
		appLocation=$(flatpak info --show-location ${appL})
		local preDate="$(stat ${appLocation} | grep Modify | awk -F': ' '{print $2}')"
		appInstallDate=$(convertDate ${preDate})

		appMetadata=$(flatpak info --show-metadata ${appL})
		appProvides=$(echo "$appMetadata" | grep command= | awk -F'=' '{print $2}')
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
	#TODO graceful wraparound
	echo -e "${bold}Description	:${normal} $appDescription"
	echo -e "${bold}Architecture	:${normal} $appArch"
	echo -e "${bold}URL		:${normal} $appUrl"
	echo -e "${bold}Licenses	:${normal} $appLicense"

	if [ -z "${appCollection}" ]; then appCollection="$appNONE"; fi
	echo -e "${bold}Groups		:${normal} $appCollection"	#change?
	
	if [ -z "$appProvides" ]; then appProvides="$appNONE"; fi
	echo -e "${bold}Provides	:${normal} $appProvides"

	local appDepends="flatpak"
	if [ -n "${appRuntime}" ]; then appDepends+="${SPACER}${appRuntime}"; fi
	echo -e "${bold}Depends On	:${normal} $appDepends"

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

flatpakArr=()	# all installed flatpaks in the format: appid/arch/branch (extended appID)
initArr;
flatpakFullList="" # big list of the installed flatpak

# get the extended appID for the provided string (returns the result to programArr)
searchAppID_local ${program}

case "$1" in
	-Q)
		if [ -z "${programArr[@]}" ] && [[ $pacReturn = 1 ]]; then
			echo -e ${err_pacNotFound}
			exit 1
		fi
		for app in ${programArr[@]}; do
			getAppInfo ${app};
			if [ -n "$COLOR" ]; then
				echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
			else
				echo -e "${appID} (${appName}) ${appVersion} (${appBranch})"
			fi
		done
		;;
	-Q*) 
		# Qi option
		if [[ "$1" == *"i"* ]]; then
			if [ -z "${programArr[@]}" ] && [[ $pacReturn = 1 ]]; then
				echo -e ${err_pacNotFound}
				exit 1
			fi
			for app in ${programArr[@]}; do
				printAppInfo ${app}
				echo ""
			done
		fi
		;;
	-S*)
		#dev
		# old (slower) variant
	#	preapp=$(flatpak --columns=app,branch search ${programArr[0]} | awk -F'\n' '{print $1}')
	#	IFS=$'\t' read -ra appArr <<< ${preapp}
	#	app="${appArr[0]}//${appArr[1]}"
	#	getAppInfo ${app};
	#	echo -e "${bold}${appID} (${appName}) ${green}${appVersion} (${appBranch})${normal}"
		;;
	-R*)
		;;
	-F*)
		;;
	-V*|--version)
		if [ -n "$WRAP" ]; then
			echo "${SPACE_VERSION} ---"
			echo "${SPACE_VERSION} "
		fi
		echo "${SPACE_VERSION} Pacpak v$VERSION - MIT License"
		echo "${SPACE_VERSION} Copyright (C) 2025 Elia Nitsche"
		;;
esac
