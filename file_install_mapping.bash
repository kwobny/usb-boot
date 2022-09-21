# This file is a bash script that is sourced by the install script.
# This file defines what and how files are installed onto this machine.
# It uses the functions defined in the install script to do this.
# The current directory is the directory of the install script.

install_file -d update_usb_boot /usr/local/bin/
install_file -d kexec_into_real_kernel /etc/usb-boot/
install_to_directory mkinitcpio_hooks/* /etc/initcpio/install/
install_file -d usb-boot.preset /etc/mkinitcpio.d/
install_file -f mkinitcpio_image_filters/add_microcode \
    /usr/local/bin/embed_microcode_into_initramfs

install_file -f rust/target/release/usb_boot /usr/local/bin/usbbootmgr
install_file -f usbbootmgr-completion.bash /usr/local/share/bash-completion/completions/usbbootmgr
