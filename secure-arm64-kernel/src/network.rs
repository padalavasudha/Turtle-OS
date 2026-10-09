use core::fmt;

#[derive(Debug, Copy, Clone)]
pub struct TcpSocket {
    pub port: u16,
    pub state: TcpState,
    pub rx_buffer: [u8; 256],
    pub tx_buffer: [u8; 256],
    pub rx_len: usize,
    pub tx_len: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TcpState {
    Closed,
    Listen,
    Established,
    ClosedWait,
}

pub struct NetworkStack {
    sockets: [Option<TcpSocket>; 4],
    socket_count: usize,
}

impl NetworkStack {
    pub fn new() -> Self {
        NetworkStack {
            sockets: [None; 4],
            socket_count: 0,
        }
    }

    pub fn create_socket(&mut self, port: u16) -> Result<usize, NetworkError> {
        if self.socket_count >= 4 {
            return Err(NetworkError::TooManySockets);
        }

        let socket = TcpSocket {
            port,
            state: TcpState::Listen,
            rx_buffer: [0; 256],
            tx_buffer: [0; 256],
            rx_len: 0,
            tx_len: 0,
        };

        let idx = self.socket_count;
        self.sockets[idx] = Some(socket);
        self.socket_count += 1;

        Ok(idx)
    }

    pub fn write_socket(&mut self, socket_id: usize, data: &[u8]) -> Result<(), NetworkError> {
        let socket = self.sockets[socket_id].as_mut().ok_or(NetworkError::SocketNotFound)?;

        if socket.tx_len + data.len() > 256 {
            return Err(NetworkError::BufferFull);
        }

        for (i, &byte) in data.iter().enumerate() {
            socket.tx_buffer[socket.tx_len + i] = byte;
        }
        socket.tx_len += data.len();

        Ok(())
    }

    pub fn read_socket(&mut self, socket_id: usize, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        let socket = self.sockets[socket_id].as_mut().ok_or(NetworkError::SocketNotFound)?;

        let len = if buffer.len() < socket.rx_len {
            buffer.len()
        } else {
            socket.rx_len
        };

        for i in 0..len {
            buffer[i] = socket.rx_buffer[i];
        }

        socket.rx_len = 0;

        Ok(len)
    }

    pub fn socket_state(&self, socket_id: usize) -> Result<TcpState, NetworkError> {
        self.sockets[socket_id]
            .as_ref()
            .map(|s| s.state)
            .ok_or(NetworkError::SocketNotFound)
    }

    pub fn close_socket(&mut self, socket_id: usize) -> Result<(), NetworkError> {
        let socket = self.sockets[socket_id].as_mut().ok_or(NetworkError::SocketNotFound)?;
        socket.state = TcpState::Closed;
        Ok(())
    }

    pub fn socket_count(&self) -> usize {
        self.socket_count
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NetworkError {
    SocketNotFound,
    TooManySockets,
    BufferFull,
    InvalidData,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::SocketNotFound => write!(f, "Socket not found"),
            NetworkError::TooManySockets => write!(f, "Too many sockets"),
            NetworkError::BufferFull => write!(f, "Buffer full"),
            NetworkError::InvalidData => write!(f, "Invalid data"),
        }
    }
}
