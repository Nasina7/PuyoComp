use std::collections::VecDeque;

pub struct CompressInstance {
    init_data: Vec<u8>,
    init_ind: usize,

    comp_buf: [Option<u8>; 0x100],
    comp_ind: usize,

    comp_buf_bak: [Option<u8>; 0x100],
    comp_ind_bak: usize,

    out_buf: Vec<u8>,
}

impl CompressInstance {
    pub fn new(filename: &str) -> Self {
        Self {
            init_data: std::fs::read(filename).unwrap(),
            init_ind: 0,

            comp_buf: [Option::None; 0x100],
            comp_ind: 0,

            comp_buf_bak: [Option::None; 0x100],
            comp_ind_bak: 0,

            out_buf: Vec::new(),
        }
    }

    // Function to read data from the decompressed file
    fn read_init(&mut self) -> u8 {
        let a = self.init_data[self.init_ind];
        self.init_ind += 1;
        a
    }

    // Writes data to the output file
    fn write_out(&mut self, val: u8) {
        self.out_buf.push(val);
    }
    
    // Takes in an array and checks to see if it can be found inside the compression buffer.
    // Returns an index if it is found, and -1 if it isn't.
    fn find_in_buf(&mut self, a: &VecDeque<u8>) -> i32 {
        // Do a backup of the buffer before we touch anything.  We'll be modifying it a lot on the fly here, and we need to be
        // able to revert the changes later.
        self.backup_buf();

        // Start at the current index into the compression buffer.
        let mut i: u16 = self.comp_ind_bak as u16;
        
        // Loop until we've looped all the way around to the original value again.
        'main_loop: while i < (0x100 | self.comp_ind_bak as u16) {
            // Reload the original buffer
            self.restore_buf();

            // Loop for the length of the inputted array.
            for x in 0..a.len() {
                match self.read_buf(i as usize + x) {
                    Option::Some(val) => {
                        // If the index contains an initialized value, check to see if it's the value we expect.
                        // If so, update the compression buffer, and if not, continue to the next index.
                        if val != a[x] {
                            i += 1;
                            continue 'main_loop;
                        } else {
                            self.write_buf(val);
                        }
                    },
                    Option::None => {
                        // The value grabbed from the decompression buffer wasn't initialized, so go to the next index.
                        i += 1;
                        continue 'main_loop;
                    }
                }
            }
            // Restore changes to the buffer and return the index at which the inputted array was first found.
            self.restore_buf();
            return (i & 0xFF) as i32
        }
        // Inputted array was not found, so restore the changes to the buffer and return -1.
        self.restore_buf();
        -1
    } 

    // Read from the compression buffer
    fn read_buf(&self, ind: usize) -> Option<u8> {
        self.comp_buf[ind & 0xFF]
    }

    // Write to the compression buffer
    fn write_buf(&mut self, val: u8) {
        self.comp_buf[self.comp_ind] = Option::Some(val);
        self.comp_ind += 1;
        self.comp_ind &= 0xFF;
    }

    // Backup the compression buffer
    fn backup_buf(&mut self) {
        self.comp_buf_bak.copy_from_slice(&self.comp_buf);
        self.comp_ind_bak = self.comp_ind;
    }

    // Restore the compression buffer
    fn restore_buf(&mut self) {
        self.comp_buf.copy_from_slice(&self.comp_buf_bak);
        self.comp_ind = self.comp_ind_bak;
    }

    // Determine what the next command in the compressed file will be.
    // v2 will always contain one byte when the function starts.
    // A return of 0 indicates a run command, while a return of 1 indicates a cache command.
    fn determine_next_command(&mut self, v2: &VecDeque<u8>) -> u32 {
        // Check to see if the data in v2 can be found in the buffer.
        let mut v3 = v2.clone();
        if self.find_in_buf(&v3) == -1 {
            // If the data cannot be found, do a run
            return 0;
        } else {
            // The byte in v2 can be found, so now check to see if there is enough uncompressed data left for a cache command.
            // (Cache commands have a minimum length of 3 bytes.)
            if self.init_ind + 2 >= self.init_data.len() {
                // Ran out of data, do a run for the rest.
                return 0;
            }

            // If the data could be found, and the bounds check passed, then do an initial check to see if the next two bytes along with this one
            // are found in the table.
            v3.push_back(self.init_data[self.init_ind+1]);
            v3.push_back(self.init_data[self.init_ind+2]);
            if self.find_in_buf(&v3) != -1 {
                // If they can be found, end the run, and initialize a normal cache command.
                return 1;
            } else {
                // If they can't be found, do a run
                return 0;
            }
        }
    }

    // Main loop to compress the data.
    pub fn compress(&mut self) -> Vec<u8> {
        // 0 = Run, 1 = Cache
        let mut next_command = 0;

        // While there is still data left to compress
        while self.init_ind < self.init_data.len() {
            // Initialize two Vectors for later usage
            let mut v: VecDeque<u8> = VecDeque::new();
            let mut v2: VecDeque<u8> = VecDeque::new();

            // If the next command is a run
            if next_command == 0 {
                loop {
                    // Push a byte into V and the buffer (at this point, this byte is confirmed not to be a part of a cache command)
                    v.push_back(self.read_init());
                    self.write_buf(v[v.len()-1]);

                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end the run.
                        break;
                    }

                    // Run commands have a maximum length of 0x7F.
                    if v.len() == 0x7F {
                        // Time to end the run.
                        break;
                    }

                    // Grab the next byte.
                    let next_byte = self.init_data[self.init_ind];

                    // Check to see if the byte can match something in the table.
                    v2 = VecDeque::new();
                    v2.push_back(next_byte);

                    // If we're still doing a run command, loop.  Otherwise, break out.
                    next_command = self.determine_next_command(&v2);
                    if next_command == 0 {
                        continue;
                    } else {
                        break;
                    }
                }

                // Write the run command to the file.
                self.write_out(v.len() as u8);
                for i in v.iter() {
                    self.write_out(*i);
                }
            } else if next_command == 1 {
                // Entering this assumes we found at least one byte in the table

                // First, we pull one byte.
                v.push_back(self.read_init());

                // Define a bool that lets us break out of this loop without removing an extra value from the buffer.
                let mut remove_value = true;
                
                // Next, we loop until we can no longer match something in the table
                loop {
                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end the cache.  Since we end unexpectedly, we don't want to remove a value from the array of bytes
                        // that the cache uses.
                        remove_value = false;
                        break;
                    }

                    // Load the next byte onto the cache run
                    v.push_back(self.read_init());

                    // If this array of values doesn't exist in the cache, break out of the loop.
                    if self.find_in_buf(&v) == -1 {break;}

                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end the cache.
                        remove_value = false;
                        break;
                    }

                    // Cache commands have a maximum length of 0x82
                    if v.len() >= 0x82 {
                        // Time to end the cache.
                        remove_value = false;
                        break;
                    }
                }

                // Here, V contains one two many elements currently (if we didn't exit unexpectedly), since it still 
                // contains the incorrect byte at the end so we remove it.
                if remove_value {
                    v.remove(v.len()-1);
                    self.init_ind -= 1;
                }

                // Grab the index of the found array
                let ind = self.find_in_buf(&v);

                // Next, we need to construct the command.
                self.write_out(0x80 | (v.len() as u8 - 3)); // Command | (Length - 3)

                // The index byte is determined by taking the original cache index and subtracting it by the found index and then also subtracting by 1.
                self.write_out((self.comp_ind as u8 - ind as u8) - 1 as u8);

                // Write the cache loaded bytes to the compression buffer
                for h in v.iter() {
                    self.write_buf(*h);
                }
                
                // Bounds Check
                if self.init_ind >= self.init_data.len() {
                    // Quit to the main loop.
                    break;
                }

                // The cache buffer is already updated, so all that's left is to find the next command.
                v2.push_back(self.init_data[self.init_ind]);
                next_command = self.determine_next_command(&v2);
            }
        }

        // Output the data.
        self.write_out(0);
        self.out_buf.clone()
    }
}