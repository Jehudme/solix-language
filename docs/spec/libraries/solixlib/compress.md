# Solix Standard Library: Compression (`solix.compress`)

The `solix.compress` package provides standard buffer compression and decompression algorithms backed by `flate2`.

## 1. Class Hierarchy & Types

* **`Gzip`**: Gzip compression format encoder and decoder.
* **`Zlib`**: Zlib compression format encoder and decoder.
* **`Deflate`**: Raw DEFLATE compression format encoder and decoder.
* **`CompressionException`**: Thrown when decompression fails on invalid buffers (`[E0334]`).

## 2. API Reference

```solix
namespace solix.compress;

public class Gzip {
    public static char[] compress(char[] data);
    public static char[] decompress(char[] data);
    public static char[] compress_string(string text);
    public static string decompress_to_string(char[] data);
}

public class Zlib {
    public static char[] compress(char[] data);
    public static char[] decompress(char[] data);
    public static char[] compress_string(string text);
    public static string decompress_to_string(char[] data);
}

public class Deflate {
    public static char[] compress(char[] data);
    public static char[] decompress(char[] data);
    public static char[] compress_string(string text);
    public static string decompress_to_string(char[] data);
}
```
