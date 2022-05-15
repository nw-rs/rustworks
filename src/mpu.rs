// pub fn init_mpu(mpu: &mut cortex_m::peripheral::MPU) {
//     use rbar::*;
//     use rasr::*;

//     unsafe {
//         // Disable MPU and clear the control register
//         mpu.ctrl.write(0);

//         mpu.rnr.write(0);
//         mpu.rbar.write(Address::conv(0x60000000));
//         mpu.rasr.write(RegionSize::_256MB | AccessPermission::NO_ACCESS | XN::ENABLE | 2 << Tex::OFFSET | ENABLE);

//         mpu.rnr.write(1);
//         mpu.rbar.write(Address::conv(0x60000000));
//         mpu.rasr.write(RegionSize::_32B | XN::ENABLE | AccessPermission::RW | 2 << Tex::OFFSET | ENABLE);

//         mpu.rnr.write(2);
//         mpu.rbar.write(Address::conv(0x60000000+0x20000));
//         mpu.rasr.write(RegionSize::_32B | XN::ENABLE | AccessPermission::RW | 2 << Tex::OFFSET | ENABLE);

//         // QUADSPI
//         mpu.rnr.write(3);
//         mpu.rbar.write(Address::conv(0x90000000));
//         mpu.rasr.write(RegionSize::_256MB | AccessPermission::NO_ACCESS | XN::ENABLE | ENABLE);

//         mpu.rnr.write(4);
//         mpu.rbar.write(Address::conv(0x90000000));
//         mpu.rasr.write(RegionSize::_8MB | AccessPermission::RW | C::ENABLE | ENABLE);

//         let mut sector = 5;
//         while sector < 8 {
//             mpu.rnr.write(sector);
//             mpu.rbar.write(Address::conv(0));
//             mpu.rasr.write(0);
//             sector += 1;
//         }
        
//         assert!(sector == 8);

//         // Enable MPU
//         mpu.ctrl.write(ctrl::PRIVDEFENA | ctrl::ENABLE);
//     }
// }

pub fn init_mpu(mpu: &mut cortex_m::peripheral::MPU) {
    unsafe {
        const FULL_ACCESS: u32 = 0b011 << 24;
        const SIZE_512MB: u32 = 28 << 1;
        const SIZE_8MB: u32 = 22 << 1;
        const DEVICE_SHARED: u32 = 0b000001 << 16;
        const NORMAL_SHARED: u32 = 0b000110 << 16;

        // Flash
        mpu.rnr.write(0);
        mpu.rbar.write(0x0000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // SRAM
        mpu.rnr.write(1);
        mpu.rbar.write(0x2000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | NORMAL_SHARED | 1);

        // Peripherals
        mpu.rnr.write(2);
        mpu.rbar.write(0x4000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(3);
        mpu.rbar.write(0x6000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(4);
        mpu.rbar.write(0xA000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // Core peripherals
        mpu.rnr.write(5);
        mpu.rbar.write(0xE000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // QSPI
        mpu.rnr.write(6);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(27 << 1 | 1 << 28 | 1);

        mpu.rnr.write(7);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_8MB | DEVICE_SHARED | 1);

        // Enable MPU
        mpu.ctrl.write(1);
    }
}

pub fn init_mpu_bootloader(mpu: &mut cortex_m::peripheral::MPU) {
    unsafe {
        mpu.ctrl.write(0);

        mpu.rnr.write(7);
        mpu.rbar.write(rbar::Address::conv(0x90000000));
        mpu.rasr.write(rasr::ENABLE);

        mpu.ctrl.write(1);
    }
}

#[allow(unused)]
mod ctrl {
    pub const ENABLE: u32 = 1;
    pub const PRIVDEFENA: u32 = 4;
    pub const HFNMIENA: u32 = 2;
}

#[allow(unused)]
mod rasr {
    pub const ENABLE: u32 = 1;
    pub struct XN;
    pub struct AccessPermission;
    pub struct RegionSize;
    pub struct Tex;
    pub struct S;
    pub struct C;
    pub struct B;
    pub struct Srd;
    pub struct Enable;

    impl XN {
        pub const OFFSET: u32 = 28;

        pub const ENABLE: u32 = 1 << Self::OFFSET;
    }

    impl AccessPermission {
        pub const OFFSET: u32 = 24;

        pub const NO_ACCESS: u32 = 0 << Self::OFFSET;
        pub const RW: u32 = 3 << Self::OFFSET;
        pub const RO: u32 = 6 << Self::OFFSET;
        pub const PRIVILEGED_RW: u32 = 1 << Self::OFFSET;
        pub const PRIVILEGED_RW_UNPRIVILEGED_RO: u32 = 2 << Self::OFFSET;
        pub const PRIVILEGED_RO: u32 = 5 << Self::OFFSET;
    }

    impl RegionSize {
        pub const OFFSET: u32 = 1;

        pub const _32B: u32 = 4 << Self::OFFSET;
        pub const _64B: u32 = 5 << Self::OFFSET;
        pub const _128B: u32 = 6 << Self::OFFSET;
        pub const _1KB: u32 = 9 << Self::OFFSET;
        pub const _64KB: u32 = 15 << Self::OFFSET;
        pub const _1MB: u32 = 19 << Self::OFFSET;
        pub const _2MB: u32 = 20 << Self::OFFSET;
        pub const _4MB: u32 = 21 << Self::OFFSET;
        pub const _8MB: u32 = 22 << Self::OFFSET;
        pub const _32MB: u32 = 24 << Self::OFFSET;
        pub const _256MB: u32 = 27 << Self::OFFSET;
        pub const _1GB: u32 = 29 << Self::OFFSET;
        pub const _4GB: u32 = 31 << Self::OFFSET;
    }

    impl Tex {
        pub const OFFSET: u32 = 19;
    }
    
    impl S {
        pub const OFFSET: u32 = 18;

        pub const ENABLE: u32 = 1 << Self::OFFSET;
    }

    impl C {
        pub const OFFSET: u32 = 17;

        pub const ENABLE: u32 = 1 << Self::OFFSET;
    }

    impl B {
        pub const OFFSET: u32 = 16;

        pub const ENABLE: u32 = 1 << Self::OFFSET;
    }

    impl Srd {
        pub const OFFSET: u32 = 8;
    }
}

#[allow(unused)]
mod rbar {
    pub struct Address;
    pub struct Valid;
    pub struct Region;

    impl Address {
        pub const OFFSET: u32 = 5;

        pub fn conv(address: u32) -> u32 {
            (address >> 5) << Self::OFFSET
        }
    }

    impl Valid {
        pub const OFFSET: u32 = 4;
    }

    impl Region {
        pub const OFFSET: u32 = 0;
    }
}