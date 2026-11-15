import struct
import zlib

def create_png(width, height, color=(255, 85, 0, 255)):
    png_signature = b'\x89PNG\r\n\x1a\n'
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff
    ihdr_chunk = struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc)
    row = b'\x00' + bytes([color[0], color[1], color[2], color[3]]) * width
    raw_data = row * height
    compressed = zlib.compress(raw_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed) & 0xffffffff
    idat_chunk = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc)
    iend_crc = zlib.crc32(b'IEND') & 0xffffffff
    iend_chunk = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc)
    return png_signature + ihdr_chunk + idat_chunk + iend_chunk

# Create ICO file with multiple sizes
png_256 = create_png(256, 256)
png_128 = create_png(128, 128)
png_64 = create_png(64, 64)
png_48 = create_png(48, 48)
png_32 = create_png(32, 32)
png_16 = create_png(16, 16)

images = [
    (256, 256, png_256),
    (128, 128, png_128),
    (64, 64, png_64),
    (48, 48, png_48),
    (32, 32, png_32),
    (16, 16, png_16),
]

with open('icon.ico', 'wb') as f:
    # ICO header: reserved (2), type (2), count (2)
    f.write(struct.pack('<HHH', 0, 1, len(images)))
    
    # Calculate offsets
    header_size = 6 + len(images) * 16
    offset = header_size
    
    # Write directory entries
    for w, h, png_data in images:
        # Width, height (0 means 256), color count, reserved, planes, bit count, size, offset
        f.write(struct.pack('<BBBBHHII', 
            w if w < 256 else 0, 
            h if h < 256 else 0, 
            0, 0, 1, 32, len(png_data), offset))
        offset += len(png_data)
    
    # Write image data
    for _, _, png_data in images:
        f.write(png_data)

print('Created icon.ico with multiple sizes')

# Create ICNS placeholder
with open('icon.icns', 'wb') as f:
    f.write(b'icns')
    # Calculate total size
    png_data = create_png(256, 256)
    total_size = 8 + 8 + len(png_data)
    f.write(struct.pack('>I', total_size))
    f.write(b'it32')  # 256x256 32-bit icon
    f.write(struct.pack('>I', len(png_data) + 8))
    f.write(png_data)
print('Created icon.icns')

