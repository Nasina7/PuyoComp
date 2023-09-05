pub struct DecompressInstance {
    init_data: Vec<u8>,
    final_data: Vec<u8>,
    
    arr1: Vec<u8>,
    arr2: Vec<u8>,
    arr1_ind: u32,
    arr2_ind: u32,
    cmd: u32,
    arr1_calcind: u32,
    data: u32,

    init_ptr: usize,
}

impl DecompressInstance {
    pub fn new(filename: &str) -> Self {
        Self {
            init_data: std::fs::read(filename).unwrap(),
            final_data: Vec::new(),
            arr1: vec![0; 0x100],
            arr2: vec![0; 0x4],
            arr1_ind: 0,
            arr2_ind: 0,
            cmd: 0,
            arr1_calcind: 0,
            data: 0,
            init_ptr: 0,
        }
    }

    // This command will load a series of bytes following the command byte.
    fn cmd_run (&mut self) {
        // Get the length of the run command from the command byte.
        self.cmd &= 0x007F;
        self.cmd -= 1;

        // Loop until we reach the end of the command.
        loop {
            // Grab a byte of data from the compressed data
            self.data = self.init_data[self.init_ptr] as u32;
            self.init_ptr += 1;

            // Store it in a 32-bit buffer.
            self.arr2[self.arr2_ind as usize] = self.data as u8;
            self.arr2_ind = (self.arr2_ind + 1) & 0xFF;

            // If the 32-bit buffer is full, output the data to "vram".
            if self.arr2_ind & 0x4 != 0 {
                self.arr2_ind = 0;
                for byte in self.arr2.iter() {
                    self.final_data.push(*byte);
                }
            }

            // Write the grabbed byte to the decompression buffer
            self.arr1[self.arr1_ind as usize] = self.data as u8;
            self.arr1_ind = (self.arr1_ind + 1) & 0xFF;

            // Decrease the length, and check if the command is done.
            self.cmd -= 1;
            if self.cmd == 0xFFFFFFFF {
                break;
            }
        }
    }

    // This command will load data from the decompression buffer, rather than the compressed file.
    fn cmd_cache(&mut self) {
        // Grab the number of bytes to load from the cache.
        self.cmd &= 0x007F;
        self.cmd += 2;

        // The second byte of the command determines where the data will begin being loaded from the buffer.
        // This requires a bit of calculation, as it isn't as straightforward as you would expect.
        self.arr1_calcind = self.arr1_ind & 0xFFFF;
        self.arr1_calcind = (self.arr1_calcind - self.init_data[self.init_ptr] as u32) & 0xFF;
        self.init_ptr += 1;
        self.arr1_calcind = (self.arr1_calcind - 1) & 0xFF;

        // Load data from the buffer until the length of the command runs out
        loop {
            // Load from the decompression buffer
            self.data = self.arr1[self.arr1_calcind as usize] as u32;
            
            // Store to the 32-bit output buffer
            self.arr2[self.arr2_ind as usize] = self.data as u8;
            self.arr2_ind = (self.arr2_ind + 1) & 0xFF;

            // If the output buffer is full, write the data to "vram"
            if self.arr2_ind & 0x4 != 0 {
                self.arr2_ind = 0;
                for byte in self.arr2.iter() {
                    self.final_data.push(*byte);
                }
            }

            // Update the decompression buffer with the newly loaded data
            self.arr1[self.arr1_ind as usize] = self.data as u8;
            self.arr1_ind = (self.arr1_ind + 1) & 0xFF;

            // Increase the index we load from, decrease the length of the command, and if we're done, break out of the command.
            self.arr1_calcind = (self.arr1_calcind + 1) & 0xFF;
            self.cmd -= 1;
            if self.cmd == 0xFFFFFFFF {
                break;
            }
        }
    }

    pub fn decompress(&mut self) -> &Vec<u8> {
        loop {
            // Get the current command from the compressed data
            self.cmd = self.init_data[self.init_ptr] as u32;
            self.init_ptr += 1;

            // If the command's highest bit is set, then it is a cache command, otherwise, it's a run command.
            // If the command is 0x00, then we've reached the end of the file, so we break out of the loop.
            if (self.cmd & 0x80) != 0 {
                self.cmd_cache();
            } else if (self.cmd & 0xFF) != 0 {
                self.cmd_run();
            } else {
                break;
            }
        }

        // Return the decompressed data.
        &self.final_data
    }
}
