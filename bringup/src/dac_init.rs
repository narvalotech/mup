use crate::I2CResources;

pub async fn test_dac(_rd: DacResources) {
    // Initialization sequence: Power to I2S PCM playback
    // 1. Apply power, assert RESET
    // 2. Wait 1.5ms
    //
    // 3. Configure XTAL driver [CHANGE TO EXT MCLK]
    // 4. Configure XTAL bias
    //
    // 5. Clear interrupts
    //    [F0000] read reg
    //
    // 6. [F0010] Enable XTAL interrupts = 0xE7
    // 7. [20000] Start XTAL = 0xF6
    //
    // 8. Configure ASP interface
    // 9. [1000B] Set ASP sample rate. 44.1kHz = 0x01
    // 10. [1000C] Set ASP sample bit size. 32b = 0x04
    // 11. ASP numerator (LE)
    //     [40010] = 0x01
    //     [40011] = 0x00
    // 12. ASP denominator (LE)
    //     [40012] = 0x08
    //     [40013] = 0x00
    // 13. ASP LRCK high time (LE)
    //     [40014] = 0x1F
    //     [40015] = 0x00
    // 14. ASP LRCK period (LE)
    //     [40016] = 0x3F
    //     [40017] = 0x00
    // 15. Configure ASP clock [CHANGE TO SLAVE]
    //     [40018] = 0x1C is master
    // 16. Configure ASP frame (for I2S input)
    //     [40019] = 0x0A
    // 17. Set ASP channel location
    //     [50000] = 0x00 (ch1 on sclk0)
    //     [50001] = 0x00 (ch2 on sclk0)
    // 18. Set channel size & enable
    //     [5000A] = 0x07 (size 32b)
    //     [5000B] = 0x0F (size 32b)
    //
    // 19. Configure PCM interface. HPF used. De-emphasis off.
    // 20. [90000] Enable HPF = 0x02
    // 21. [90001] Set vol channel B = 0x00 (0dB)
    // 22. [90002] Set vol channel A = 0x00 (0dB)
    // 23. Configure PCM path signal control
    //     [90003] = 0xEC
    //     [90004] = 0x00
    //
    // 24. Configure HP
    // 25. [B0000] Class H amplifier control = 0x1E
    // 26. [80000] Set HP output to full-scale = 0x30
    // 27. [D0000] Configure HP detect = 0x04
    // 28. [D0000] Enable HP detect = 0xC4
    //
    // 29. Enable interrupts
    // 30. [F0000] read reg (clears IRQ)
    // 31. [F0010] Enable headphone detect IRQ = 0x87
    // 32. [F0011] Enable ASP IRQ = 0x07
    // 33. [F0000] Wait for IRQ (XTAL_READ_INT=1) [CHANGE]
    // 34. [10006] Switch MCLK src to XTAL = 0x04 [CHANGE]
    // 35. Wait 150us
    // 36. [1000D] Enable ASP clocks = 0x02
    //
    // 37. Power up HP (see below)

    // Headphone power up sequence
    // 1. Pop-free power-up settings
    //    [10010] = 0x99
    //    [80032] = 0x20
    // 2. [20000] Power up ASP PDN_ASP = 0
    // 3. [20000] Power up amplifier PDN_HP = 0
    // 4. Wait 12ms
    // 5. Restore default settings
    //    [80032] = 0x00
    //    [10010] = 0x00

    // Headphone power down sequence
    // 1. [F0010] Enable PDN_DONE interrupt
    // 2. [20000] Power down amplifier PDN_HP = 1
    // 3. [F0000] Wait for PDN_DONE_INT
    // 4. [20000] Power down ASP PDN_ASP = 1
}
