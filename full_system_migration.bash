# This bash script file contains steps, commands, and notes for
# fully backing up and migrating the system to another installation.
# IT IS NOT MEANT TO BE RUN.

echo 'Do not run this as a script.'
exit 1

# The backup is split into multiple steps. The process starts with the
# list of all files under '/'. This list is sort of "piped" into each
# of the steps.
# Each step handles a certain part of the files piped into it,
# and passes the list of remaining files onto the next step.
# 1. Exclude special filesystems/directories like /sys, /proc, /dev, etc.
# 2. Backup files through package list.
# 3. Backup files in tar archive.
# 4. Files not backed up.
# Each step should have a component that takes a list of filenames
# through stdin, filters it, then outputs the unhandled/remaining
# filenames to stdout.

special_directories_regex='^/(sys|srv|proc|dev|tmp|run)|.*/\.ccache/.*'
tar_archive_files_regex='^/(etc|boot|efi|home)/.*'

find / -regextype posix-extended -regex "$special_directories_regex" -prune -o -print | \
    while read -r filename; do
        pacman -Qo "$filename" &>/dev/null || echo "$filename"
    done | \
    grep -Ev "$tar_archive_files_regex"
