#!/bin/bash
cd /sys/kernel/config/usb_gadget/
mkdir -p gadget
cd gadget

# https://devicehunt.com/all-usb-vendors
echo 0x046D > idVendor # Logitech, Inc. Vendor
echo 0xC08B > idProduct # G502 SE HERO Gaming Mouse

echo 0x0100 > bcdDevice # v1.0.0
echo 0x0310 > bcdUSB # USB3.1

mkdir -p strings/0x409
echo "0E6D395D3333" > strings/0x409/serialnumber
echo "Logitech" > strings/0x409/manufacturer
echo "G502 SE HERO Gaming Mouse" > strings/0x409/product

mkdir -p configs/c.1/strings/0x409
echo 300 > configs/c.1/MaxPower

# Add functions here

mkdir -p functions/hid.usb0
echo 0 > functions/hid.usb0/protocol
echo 0 > functions/hid.usb0/subclass
echo 133 > functions/hid.usb0/report_length

echo -ne \\x05\\x01\\x09\\x02\\xA1\\x01\\x09\\x01\\xA1\\x00\\x85\\x01\\x05\\x09\\x19\\x01\\x29\\x05\\x15\\x00\\x25\\x01\\x95\\x05\\x75\\x01\\x81\\x02\\x95\\x01\\x75\\x03\\x81\\x03\\x05\\x01\\x09\\x30\\x09\\x31\\x16\\x01\\x80\\x26\\xFF\\x7F\\x75\\x10\\x95\\x02\\x81\\x06\\x09\\x38\\x15\\x81\\x25\\x7F\\x75\\x08\\x95\\x01\\x81\\x06\\xC0\\xC0\\x05\\x01\\x09\\x06\\xA1\\x01\\x85\\x02\\x05\\x07\\x19\\xE0\\x29\\xE7\\x15\\x00\\x25\\x01\\x75\\x01\\x95\\x08\\x81\\x02\\x75\\x08\\x95\\x01\\x81\\x01\\x75\\x01\\x95\\x03\\x05\\x08\\x19\\x01\\x29\\x03\\x91\\x02\\x75\\x01\\x95\\x05\\x91\\x01\\x75\\x08\\x95\\x06\\x15\\x00\\x26\\xFF\\x00\\x05\\x07\\x19\\x00\\x2A\\xFF\\x00\\x81\\x00\\xC0 > functions/hid.usb0/report_desc

ln -s functions/hid.usb0 configs/c.1/

# End functions

ls /sys/class/udc > UDC
