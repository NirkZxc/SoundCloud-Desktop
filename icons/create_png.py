import struct
import zlib
import base64

def create_png(width, height, color=(255, 85, 0, 255)):
    """Create a minimal valid PNG file"""
    # PNG signature
    png_signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff
    ihdr_chunk = struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc)
    
    # IDAT chunk - simple solid color image
    # Each row: filter byte (0) + pixel data
    row = b'\x00' + bytes([color[0], color[1], color[2], color[3]]) * width
    raw_data = row * height
    compressed = zlib.compress(raw_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed) & 0xffffffff
    idat_chunk = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc)
    
    # IEND chunk
    iend_crc = zlib.crc32(b'IEND') & 0xffffffff
    iend_chunk = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc)
    
    return png_signature + ihdr_chunk + idat_chunk + iend_chunk

# Create PNGs for different sizes
sizes = [32, 128, 256]
for size in sizes:
    png_data = create_png(size, size)
    with open(f'{size}x{size}.png', 'wb') as f:
        f.write(png_data)
    print(f'Created {size}x{size}.png')

# Create 128x128@2x.png (same as 256x256)
with open('128x128@2x.png', 'wb') as f:
    f.write(create_png(256, 256))
print('Created 128x128@2x.png')

# Create ICO file (simplified - just use 256x256 as ICO)
# ICO format is more complex, let's skip for now and create a simple one
with open('icon.ico', 'wb') as f:
    # ICO header
    f.write(struct.pack('<HHH', 0, 1, 1))  # Reserved, type (1=ico), count
    # ICO directory entry
    png_data = create_png(256, 256)
    f.write(struct.pack('<BBBBHHII', 256, 256, 0, 0, 1, 32, len(png_data), 22))
    f.write(png_data)
print('Created icon.ico')

# Create ICNS (macOS) - simplified, just use PNG
with open('icon.icns', 'wb') as f:
    f.write(b'icns')  # Magic
    f.write(struct.pack('>I', 0))  # File size placeholder
    # This is a minimal placeholder - real ICNS is more complex
    f.write(create_png(256, 256))
print('Created icon.icns (placeholder)')
