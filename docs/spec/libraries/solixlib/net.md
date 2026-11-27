# Solix Standard Library: Networking (`solix.net`)

The `solix.net` package provides cross-platform network socket primitives and HTTP client functionality backed by the host operating system's native network stack via `std::net`.

## 1. Class Hierarchy & Types

* **`TcpListener`**: Server socket listening on a specified `host:port` address.
* **`TcpStream`**: Stream-oriented TCP network connection.
* **`UdpSocket`**: Datagram socket for connectionless UDP packet transmission.
* **`HttpResponse`**: Encapsulates HTTP status code, reason phrase, lowercase-keyed header map, and decoded body.
* **`HttpClient`**: HTTP/1.1 client supporting `get`, `get_response`, `post`, `put`, `delete`, and `request`, complete chunked/multi-buffer reading, and fail-fast scheme validation.
* **`NetworkException`**: Thrown on socket binding, connection, or I/O failure (`[E0333]`).

## 2. API Reference

```solix
package solix.net;

public class TcpListener {
    public TcpListener(int64 handle);
    ~TcpListener();
    public static TcpListener bind(string host_port);
    public static TcpListener bind(String host_port);
    public TcpStream accept();
    public void close();
}

public class TcpStream {
    public TcpStream(int64 handle);
    ~TcpStream();
    public static TcpStream connect(string host_port);
    public static TcpStream connect(String host_port);
    public static TcpStream connect_timeout(string host_port, int32 timeout_ms);
    public static TcpStream connect_timeout(String host_port, int32 timeout_ms);
    public int32 write(int8[] buffer);
    public int32 write_string(string text);
    public int32 write_string(String text);
    public int8[] read(int32 max_bytes);
    public String read_string(int32 max_bytes);
    public bool set_read_timeout(int32 timeout_ms);
    public bool set_write_timeout(int32 timeout_ms);
    public void close();
}

public class UdpSocket {
    public UdpSocket(int64 handle);
    ~UdpSocket();
    public static UdpSocket bind(string host_port);
    public static UdpSocket bind(String host_port);
    public int32 send_to(int8[] buffer, string target_address);
    public int32 send_to(int8[] buffer, String target_address);
    public int8[] recv_from(int32 max_bytes);
    public bool set_read_timeout(int32 timeout_ms);
    public bool set_write_timeout(int32 timeout_ms);
    public void close();
}

public class HttpResponse {
    public int32 status_code;
    public String reason_phrase;
    public HashMap<String, String> headers;
    public String body;

    public bool is_ok();
    public String get_header(string header_name);
    public String get_header(String header_name);
}

public class HttpClient {
    public static String get(string url);
    public static String get(String url);
    public static HttpResponse get_response(string url);
    public static HttpResponse get_response(String url);
    public static HttpResponse post(string url, string body);
    public static HttpResponse post(String url, String body);
    public static HttpResponse put(string url, string body);
    public static HttpResponse put(String url, String body);
    public static HttpResponse delete(string url);
    public static HttpResponse delete(String url);
    public static HttpResponse request(string method, string url);
    public static HttpResponse request(string method, string url, string body);
    public static HttpResponse request(
        String method,
        String url,
        String body,
        HashMap<String, String> custom_headers
    );
}
```

## 3. RAII, UTF-8 & HTTP Transport Behavior

* **Deterministic Socket Cleanup:** `TcpListener`, `TcpStream`, and `UdpSocket` implement ARC destructors (`~TcpListener()`, `~TcpStream()`, `~UdpSocket()`) that call `this.close()` automatically on scope exit, closing OS socket descriptors via `native_close`.
* **UTF-8 Text Framing:** `TcpStream::write_string` and `TcpStream::read_string` encode and decode text via `String::to_utf8_bytes()` and `String::from_utf8_bytes(int8[])`, preserving multi-byte UTF-8 sequences without narrowing characters to single signed bytes.
* **Complete HTTP/1.1 Framing & Scheme Validation:** `HttpClient` rejects `https://` URLs and unsupported schemes early with `[E0333]` (`NetworkException`), reads arbitrarily large responses iteratively in 16 KiB chunks honoring `Content-Length`, decodes `Transfer-Encoding: chunked` bodies, normalizes response header names to lowercase for `HttpResponse::get_header(...)`, and closes the underlying `TcpStream` inside a `try / finally` block.
