var searchIndex = JSON.parse('{\
"ti_sbl":{"doc":"TI Serial Bootloader Interface library","t":[13,13,13,3,4,11,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,5,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,3,3,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,12,12,12,12,12,12,12,11,11,11,11,11,11,11,11,12,12,17,17,3,11,11,12,5,12,11,11,11,5,5,12,5,11,11,11,5],"n":["CC2538","CC26X0","CC26X2","Device","Family","address_to_page","borrow","borrow","borrow_mut","borrow_mut","clone","clone_into","constants","download","eq","erase","family","flash_base","fmt","fmt","from","from","from_str","get_chip_id","get_status","into","into","invoke_bootloader","memory_read_32","new","ping","port_settings","ports","sector_erase","sector_size","send_data","set_xosc","supports_bank_erase","supports_download_crc","supports_erase","supports_run","supports_sector_erase","supports_set_ccfg","supports_set_xosc","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","util","ACK","CC2538_CMD_ERASE","CC2538_CMD_RUN","CC2538_CMD_SET_XOSC","CC26X0_CMD_BANK_ERASE","CC26X0_CMD_SECTOR_ERASE","CC26X0_CMD_SET_CCFG","CC26X2_CMD_DOWNLOAD_CRC","CMD_CRC32","CMD_DOWNLOAD","CMD_GET_CHIP_ID","CMD_GET_STATUS","CMD_MEMORY_READ","CMD_MEMORY_WRITE","CMD_PING","CMD_RESET","CMD_SEND_DATA","COMMAND_RET_FLASH_FAIL","COMMAND_RET_INVALID_ADR","COMMAND_RET_INVALID_CMD","COMMAND_RET_SUCCESS","COMMAND_RET_UNKNOWN_CMD","MAX_BYTES_PER_TRANSFER","NACK","PortInfo","PortUsbInfo","borrow","borrow","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","fmt","fmt","from","from","interface","into","into","list_all","manufacturer","name","num_if","pid","port","product","serial","to_owned","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","usb_info","vid","CCFG_SIZE","INVALID_ADDR","Transfer","borrow","borrow_mut","data","erase_flash_range","expect_ack","fmt","from","into","read_flash_size","read_ieee_address","start_address","status_code_to_str","try_from","try_into","type_id","write_flash_range"],"q":["ti_sbl","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","ti_sbl::constants","","","","","","","","","","","","","","","","","","","","","","","","ti_sbl::ports","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","ti_sbl::util","","","","","","","","","","","","","","","","","",""],"d":["CC2538 microcontrollers.","CC26x0 and CC13x0 microcontrollers.","CC26x2 and CC13x2 microcontrollers.","A TI connected device supporting the Serial Bootloader …","The type of the bootloader.","Convert a flash address to the flash page.","","","","","","","","Prepares flash programming.","","Erase. Only supported on [<code>Family::CC2538</code>].","Returns the <code>Family</code> of the device.","Flash base size.","","","","","","Read chip ID.","Get the status of the last issued command.","","","Use the DTR and RTS lines to control bootloader and the …","Read memory using 32-bit access type.","Create a new <code>Device</code> from an already opened port.","Ping the bootloader.","Default serial port settings.","","Sector erase. Only supported on [<code>Family::CC26X0</code>] and […","Sector erase size, in bytes.","Send data to be written into the flash memory.","Switch to XOSC. Only supported on [<code>Family::CC2538</code>].","Whether the device supports <code>COMMAND_BANK_ERASE</code>.","Whether the device supports <code>COMMAND_DOWNLOAD_CRC</code>.","Whether the device supports <code>COMMAND_ERASE</code>.","Whether the device supports <code>COMMAND_RUN</code>.","Whether the device supports <code>COMMAND_SECTOR_ERASE</code>.","Whether the device supports <code>COMMAND_SET_CCFG</code>.","Whether the device supports <code>COMMAND_SET_XOSC</code>.","","","","","","","","Utilities","ACK byte","","","","","","","","","","","","","","","","","","","","","","Maximum bytes per transfer, on [<code>CMD_SEND_DATA</code>] commands.","NACK byte","Information about an available serial port.","Information about USB serial ports.","","","","","","","","","","","","","Device product interface.","","","List all serial ports on the system.","Device manufacturer.","","Number of interfaces in this device.","USB Product ID.","","Device product description.","Serial number string.","","","","","","","","","","USB Vendor ID.","CC26xx/CC13xx CCFG size in bytes.","The value of an invalid IEEE/BLE address in the CCFG.","A binary data transfer","","","The data to write on the device’s flash.","Erase a flash range.","Whether we expect an ACK in return.","","","","Reads the flash size from the memory.","Read IEEE 802.15.4g MAC address.","The start address in flash of this data.","","","","","Write the flash."],"i":[1,1,1,0,0,1,2,1,2,1,1,1,0,2,1,2,2,1,2,1,2,1,1,2,2,2,1,0,2,2,2,0,0,2,1,2,2,1,1,1,1,1,1,1,1,2,1,2,1,2,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,4,3,4,3,4,3,4,3,4,3,4,4,3,4,3,4,3,4,4,3,4,4,3,4,3,4,3,4,3,4,3,4,0,0,0,5,5,5,0,5,5,5,5,0,0,5,0,5,5,5,0],"f":[null,null,null,null,null,[[["u32",15]],["u32",15]],[[]],[[]],[[]],[[]],[[],["family",4]],[[]],null,[[["u32",15]],["result",6]],[[["family",4]],["bool",15]],[[["u32",15]],["result",6]],[[],["family",4]],[[],["u32",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[["str",15]],["result",4]],[[],[["result",6],["u32",15]]],[[],[["result",6],["u8",15]]],[[]],[[]],[[["bool",15]],["result",6]],[[["u32",15]],["result",6]],[[["family",4]],["result",6]],[[],[["result",6],["bool",15]]],[[],["portsettings",3]],null,[[["u32",15]],["result",6]],[[],["u32",15]],[[],[["result",6],["bool",15]]],[[],["result",6]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[],["portinfo",3]],[[],["portusbinfo",3]],[[]],[[]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],null,[[]],[[]],[[],[["vec",3],["portinfo",3]]],null,null,null,null,null,null,null,[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,null,null,null,null,[[]],[[]],null,[[["u32",15],["device",3]],["result",6]],null,[[["formatter",3]],["result",6]],[[]],[[]],[[["device",3]],[["result",6],["u32",15]]],[[["device",3]],["result",6]],null,[[["u8",15]],["str",15]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[["device",3]],["result",6]]],"p":[[4,"Family"],[3,"Device"],[3,"PortInfo"],[3,"PortUsbInfo"],[3,"Transfer"]]},\
"ti_sbl_prog":{"doc":"","t":[17,3,12,5,12,12,12,11,11,5,12,12,0,5,11,11,0,5,5,12,11,11,11,11,3,12,12,11,11,5,12,11,11,11,5,11,11,11,12,5],"n":["DEFAULT_PORT","GlobalArgs","baudrate","baudrate_to_usize","bootloader_active_low","bootloader_inverted","bootloader_invoke","borrow","borrow_mut","cli","enable_xosc","family","flash","format_addr","from","into","list","main","opt","port","port_to_string","try_from","try_into","type_id","FlashArgs","address","binary_path","borrow","borrow_mut","flash","force","from","from_matches","into","may_overwrite_ccfg","try_from","try_into","type_id","write_erase","list"],"q":["ti_sbl_prog","","","","","","","","","","","","","","","","","","","","","","","","ti_sbl_prog::flash","","","","","","","","","","","","","","","ti_sbl_prog::list"],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Flash subcommand entry point.","","","","","","","","","",""],"i":[0,0,1,0,1,1,1,1,1,0,1,1,0,0,1,1,0,0,0,1,1,1,1,1,0,2,2,2,2,0,2,2,2,2,0,2,2,2,2,0],"f":[null,null,null,[[["baudrate",4]],["usize",15]],null,null,null,[[]],[[]],[[],["app",3]],null,null,null,[[],["string",3]],[[]],[[]],null,[[],["result",6]],[[["str",15]],["arg",3]],null,[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],null,null,null,[[]],[[]],[[["argmatches",3],["u32",15],["device",3]],["result",6]],null,[[]],[[["argmatches",3]],[["flashargs",3],["result",6]]],[[]],[[["u32",15]],["bool",15]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],null,[[],["result",6]]],"p":[[3,"GlobalArgs"],[3,"FlashArgs"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};