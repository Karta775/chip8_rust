use std::process;

pub struct Stack {
    stack: [u16;32],
    top: i8,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: [0;32],
            top: -1,
        }
    }
    pub fn push(&mut self, element: u16) {
        if self.is_full() == false {
            self.top += 1;
            self.stack[self.top as usize] = element;
        } else {
            eprintln!("Error: stack is full and cannot push");
            process::exit(1);
        }
    }
    pub fn pop(&mut self) -> u16 {
        let element: u16;

        if self.is_empty() == false {
            element = self.stack[self.top as usize];
            self.top -= 1;
            element
        } else {
            eprintln!("Error: stack is empty and cannot pop");
            process::exit(1);
        }
    }
    pub fn top(&self) -> u16 {
        if self.is_empty() == false {
            self.stack[self.top as usize]
        } else {
            eprintln!("Error: stack is empty and cannot pop");
            process::exit(1);
        }
    }
    pub fn is_full(&self) -> bool {
        match self.top {
            31 => true,
            _ => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self.top {
            -1 => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stack_push_pop() {
        let mut stack = Stack::new();
        stack.push(5);
        stack.push(7);
        assert_eq!(stack.stack[0], 5);
        assert_eq!(stack.stack[1], 7);
        assert_eq!(stack.pop(), 7);
        assert_eq!(stack.pop(), 5);
        assert_eq!(stack.top, -1);
        assert_eq!(stack.is_full(), false);
        assert_eq!(stack.is_empty(), true);
    }

    #[test]
    fn stack_is_empty() {
        let stack = Stack::new();
        assert_eq!(stack.is_empty(), true);
    }

    #[test]
    fn stack_is_full() {
        let mut stack = Stack::new();
        for idx in 0..32 {
            stack.push(idx);
        }
        assert_eq!(stack.is_full(), true);
        stack.pop();
        assert_eq!(stack.is_full(), false);
    }
}