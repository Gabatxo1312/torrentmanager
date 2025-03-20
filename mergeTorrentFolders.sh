#! /usr/bin/env bash

set -eu
set -o pipefail
shopt -s nullglob

help() {
	echo "mergeTorrentFolders.sh TORRENTSOURCE TORRENTDEST"
	echo "Migrate all torrents/magnets in a SOURCE folder to a DEST folder, renaming them by their torrentID."
	echo "Removes magnet files when the corresponding torrent file exists."
}

runMigrate() {
	if [ ! -d "$1" ]; then
		echo "Source needs to be a folder: $1"
	fi
	if [ ! -d "$2" ]; then
		echo "Destination needs to be a folder: $2"
	fi

	for file in "$1"/*.torrent; do
		id="$(qbt id "$file")"
		if [ $? -eq 0 ]; then
			newfilename="$id".torrent
			#echo $newfilename
			if [ ! -f "$2"/"$newfilename" ]; then
				cp "$file" "$2"/"$newfilename"
			fi
		else
			echo "Failed to read torrent: $file"
		fi
	done

	for file in "$1"/*.magnet; do
		magnet="$(cat "$file")"
		id="$(qbt id "$magnet")"
		if [ $? -eq 0 ]; then
			newfilename="$id".magnet
			# If no corresponding magnet/torrent, copy it
			if [ ! -f "$2"/"$id".torrent ] && [[ ! -f "$2"/"$id".magnet ]]; then
					cp "$file" "$2"/"$newfilename"
			else
				# If there is already a corresponding magnet/torrent, don't make a new copy
				# but ensure if the source/dest folders are the same, we only retain one copy
				if [[ "$1" = "$2" ]] && [[ "$file" != "$2"/"$id".magnet ]]; then
					echo "Removing magnet $file because corresponding torrent "$id" already exists"
					rm "$file"
				fi
			fi
		else
			echo "Failed to read magnet: $file"
		fi
	done
}

if [ ! $# -eq 2 ]; then
	help
	exit 1
fi

case "$1" in
	"help"|"--help"|"-h")
		help
		exit 0
		;;
	*)
		runMigrate "$@"
		;;
esac
