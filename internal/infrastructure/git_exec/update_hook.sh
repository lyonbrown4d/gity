#!/bin/sh
refname="$1"
oldrev="$2"
newrev="$3"
zero="0000000000000000000000000000000000000000"

case "
$GITY_DENY_FORCE_PUSH_REFS
" in
*"
$refname
"*)
	if [ "$oldrev" != "$zero" ] && [ "$newrev" != "$zero" ]; then
		if ! git merge-base --is-ancestor "$oldrev" "$newrev"; then
			echo "force push is not allowed for protected branch: ${refname#refs/heads/}" >&2
			exit 1
		fi
	fi
	;;
esac

exit 0
