use crate::*;

type B = Into<&[u8]>;

pub trait Socket<IFrame: B, OFrame: B> {
    fn is_reliable() -> bool;
    fn send(&self, frame: OFrame);
    fn recv(&self, frame: IFrame);
}

pub trait TcpSocket<IFrame: B, OFrame: B> {
    fn tcp_send(&self, frame: OFrame);
    fn tcp_recv(&self, frame: IFrame);
}
pub trait UdpSocket<Iframe: B, OFrame: B> {
    fn udp_send(&self, frame: OFrame);
    fn udp_recv(&self, frame: IFrame);
}
impl Socket<I, O> for TcpSocket<I, O> {
    fn is_reliable() {
        return true;
    }
    fn send(&self, frame: O) {
        self.tcp_send(frame)
    }
    fn recv(&self, frame: I) {
        self.tcp_recv(frame)
    }
}

impl Socket<I, O> for UdpSocket<I, O> {
    fn is_reliable() {
        return false;
    }
    fn send(&self, frame: O) {
        self.udp_send(frame)
    }
    fn recv(&self, frame: I) {
        self.udp_recv(frame)
    }
}