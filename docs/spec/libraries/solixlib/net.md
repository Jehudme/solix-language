# Solix Standard Library: Networking (`solix.net`)

The `solix.net` package provides cross-platform network socket primitives and HTTP client functionality backed by the host operating system's native network stack via `std::net`.

## 1. Class Hierarchy & Types

* **`TcpListener`**: Server socket listening on a specified `host:port` address.
* **`TcpStream`**: Stream-oriented TCP network connection.
* **`UdpSocket`**: Datagram socket for connectionless UDP packet transmission.
* **`HttpClient`**: Lightweight HTTP request client supporting GET and POST operations.
* **`NetworkException`**: Thrown on socket binding, connection, or I/O failure (`[E0333]`).

## 2. API Reference

```solix
namespace solix.net;

public class TcpListener {
    public static TcpListener bind(string host_port);
    public TcpStream accept();
    public void close();
}

public class TcpStream {
    public static TcpStream connect(string host_port);
    public static TcpStream connect_timeout(string host_port, int32 timeout_ms);
    public int32 write(int8[] buffer);
    public int8[] read(int32 max_bytes);
    public bool set_read_timeout(int32 timeout_ms);
    public bool set_write_timeout(int32 timeout_ms);
    public void close();
}

public class UdpSocket {
    public static UdpSocket bind(string host_port);
    public int32 send_to(int8[] buffer, string target_address);
    public int8[] recv_from(int32 max_bytes);
    public bool set_read_timeout(int32 timeout_ms);
    public bool set_write_timeout(int32 timeout_ms);
    public void close();
}

public class HttpClient {
    public static string get(string url);
    public static string post(string url, string payload, string content_type);
}
```
