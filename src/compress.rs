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

    fn read_init(&mut self) -> u8 {
        let a = self.init_data[self.init_ind];
        self.init_ind += 1;
        a
    }

    fn write_out(&mut self, val: u8) {
        self.out_buf.push(val);
    }
    
    fn find_in_buf(&mut self, a: &VecDeque<u8>) -> i32 {
        self.backup_buf();
        let mut i: u16 = self.comp_ind_bak as u16;
        'main_loop: while i < (0x100 | self.comp_ind_bak as u16) {
            self.restore_buf();
            for x in 0..a.len() {
                match self.read_buf(i as usize + x) {
                    Option::Some(val) => {
                        if val != a[x] {
                            i += 1;
                            continue 'main_loop;
                        } else {
                            self.write_buf(val);
                        }
                    },
                    Option::None => {
                        i += 1;
                        continue 'main_loop;
                    }
                }
            }
            self.restore_buf();
            return (i & 0xFF) as i32
        }
        self.restore_buf();
        -1
    } 

    fn read_buf(&self, ind: usize) -> Option<u8> {
        self.comp_buf[ind & 0xFF]
    }

    fn write_buf(&mut self, val: u8) {
        self.comp_buf[self.comp_ind] = Option::Some(val);
        self.comp_ind += 1;
        self.comp_ind &= 0xFF;
    }

    fn backup_buf(&mut self) {
        self.comp_buf_bak.copy_from_slice(&self.comp_buf);
        self.comp_ind_bak = self.comp_ind;
    }

    fn restore_buf(&mut self) {
        self.comp_buf.copy_from_slice(&self.comp_buf_bak);
        self.comp_ind = self.comp_ind_bak;
    }

    // Here, v2 is the next byte to check, while v is the current data being processed.
    // v2 will always only contain one byte.
    fn determine_next_command(&mut self, v2: &VecDeque<u8>) -> u32 {
        let mut v3 = v2.clone();
        if self.find_in_buf(&v3) == -1 {
            // If the data cannot be found, do a run
            return 0;
        } else {
            // Bounds Check for input data.
            if self.init_ind + 2 >= self.init_data.len() {
                // Ran out of data, do a run for the rest.
                return 0;
            }

            // If it can, then do an initial check to see if the next two bytes along with this one
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

    pub fn compress(&mut self) -> Vec<u8> {
        // 0 = Run, 1 = Cache
        let mut next_command = 0;

        while self.init_ind < self.init_data.len() {
            let mut v: VecDeque<u8> = VecDeque::new();
            let mut v2: VecDeque<u8> = VecDeque::new();

            if next_command == 0 {
                loop {

                    // Push the byte into V and the buffer.
                    v.push_back(self.read_init());
                    self.write_buf(v[v.len()-1]);

                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end.
                        break;
                    }

                    if v.len() == 0x7F {
                        break;
                    }

                    // Grab the next byte.
                    let next_byte = self.init_data[self.init_ind];

                    // Check to see if the byte can match something in the table.
                    v2 = VecDeque::new();
                    v2.push_back(next_byte);

                    // If we're still looping, loop.
                    next_command = self.determine_next_command(&v2);
                    if next_command == 0 {
                        continue;
                    } else {
                        break;
                    }
                }

                self.write_out(v.len() as u8);
                for i in v.iter() {
                    self.write_out(*i);
                }
            } else if next_command == 1 {
                // Normal Cache operation
                // Entering this assumes we found at least one byte in the table

                // First, we pull one byte.
                v.push_back(self.read_init());

                // Backup the cache index for later
                let mut remove_value = true;
                
                // Next, we loop until we can no longer match something in the table
                loop {
                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end.
                        remove_value = false;
                        break;
                    }

                    // Load the next byte onto the cache run
                    v.push_back(self.read_init());

                    // If this array of values doesn't exist in the cache, break out of the loop.
                    if self.find_in_buf(&v) == -1 {break;}

                    // Bounds Check
                    if self.init_ind >= self.init_data.len() {
                        // Time to end.
                        remove_value = false;
                        break;
                    }

                    if v.len() >= 0x82 {
                        remove_value = false;
                        break;
                    }
                }

                // Here, V contains one two many elements currently, since it still contains the incorrect byte at the end, so we remove it.
                if remove_value {
                    v.remove(v.len()-1);
                    self.init_ind -= 1;
                }

                // Grab the index of the found array
                let ind = self.find_in_buf(&v);

                // Next, we need to construct the command.
                self.write_out(0x80 | (v.len() as u8 - 3)); // Command | (Length - 3)

                // The index byte is determined by taking the original cache index and subtracting it by the found index and then also subtracting by 1.
                self.write_out((self.comp_ind as u8 - ind as u8) - 1 as u8); // NOTE: Does this also require a subtraction by the length of V like in main_loop_2?

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
        self.write_out(0);
        self.out_buf.clone()
    }
}