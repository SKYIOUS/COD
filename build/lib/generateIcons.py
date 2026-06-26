"""Generate all platform icon files from COD.png"""
from PIL import Image
import struct
import os
import io
import shutil

ROOT = r'C:\Users\nanda\Desktop\Github\COD'
SRC = os.path.join(ROOT, 'COD.png')

def create_ico(png_data, sizes):
    """Create .ico file from PNG data at multiple sizes"""
    # ICO header
    header = struct.pack('<HHH', 0, 1, len(sizes))
    data_offset = 6 + 16 * len(sizes)
    entries = b''
    images = []
    for w, h in sizes:
        img = Image.open(io.BytesIO(png_data)).resize((w, h), Image.LANCZOS)
        buf = io.BytesIO()
        img.save(buf, 'PNG')
        png_bytes = buf.getvalue()
        size = len(png_bytes)
        entries += struct.pack('<BBBBHHII', w if w < 256 else 0, h if h < 256 else 0, 0, 0, 1, 32, size, data_offset)
        images.append(png_bytes)
        data_offset += size
    return header + entries + b''.join(images)

def create_icns(png_path):
    """Create .icns file from a 1024x1024 PNG"""
    # .icns format: header + icon entries
    # We only embed a single ic07 (128x128) and ic09 (256x256) and ic10 (512x512) entry
    # For real usage, use iconutil on macOS
    img = Image.open(png_path)
    entries = b''
    for size, ostype in [(128, b'ic07'), (256, b'ic08'), (512, b'ic09'), (1024, b'ic10')]:
        resized = img.resize((size, size), Image.LANCZOS)
        buf = io.BytesIO()
        resized.save(buf, 'PNG')
        png_data = buf.getvalue()
        entry_size = 8 + len(png_data)
        entries += ostype + struct.pack('>I', entry_size) + png_data
    total_size = 8 + len(entries)
    return b'icns' + struct.pack('>I', total_size) + entries

def convert_icons():
    img = Image.open(SRC)
    print(f'Source image: {img.size}, mode: {img.mode}')

    # Ensure RGBA
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    png_buf = io.BytesIO()
    img.save(png_buf, 'PNG')
    png_data = png_buf.getvalue()

    # === Windows .ico ===
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (96, 96), (128, 128), (256, 256)]
    ico_data = create_ico(png_data, ico_sizes)
    win32_dir = os.path.join(ROOT, 'resources', 'win32')
    with open(os.path.join(win32_dir, 'code.ico'), 'wb') as f:
        f.write(ico_data)
    print('Generated resources/win32/code.ico')

    for w, h in [(70, 70), (150, 150)]:
        resized = img.resize((w, h), Image.LANCZOS)
        resized.save(os.path.join(win32_dir, f'code_{w}x{h}.png'))
        print(f'Generated resources/win32/code_{w}x{h}.png')

    # === Windows installer .bmp ===
    inno_dir = win32_dir
    for scale in ['100', '125', '150', '175', '200', '225', '250']:
        for prefix, size_hint in [('inno-big', 55), ('inno-small', 44)]:
            factor = int(scale) / 100
            sz = max(1, int(size_hint * factor))
            resized = img.resize((sz, sz), Image.LANCZOS)
            bmp_path = os.path.join(inno_dir, f'{prefix}-{scale}.bmp')
            # Save as 32-bit BGRA BMP
            bgra = Image.new('RGBA', resized.size)
            bgra.paste(resized, (0, 0))
            r, g, b, a = bgra.split()
            bgra = Image.merge('RGBA', (b, g, r, a))
            bgra.save(bmp_path)
            print(f'Generated {prefix}-{scale}.bmp')

    # === macOS .icns ===
    darwin_dir = os.path.join(ROOT, 'resources', 'darwin')
    icns_data = create_icns(SRC)
    with open(os.path.join(darwin_dir, 'code.icns'), 'wb') as f:
        f.write(icns_data)
    print('Generated resources/darwin/code.icns')

    # === Linux .png ===
    linux_dir = os.path.join(ROOT, 'resources', 'linux')
    for size in [64, 128, 256, 512]:
        resized = img.resize((size, size), Image.LANCZOS)
        resized.save(os.path.join(linux_dir, f'code.png'))
        print(f'Generated resources/linux/code.png')
        break  # Only need one copy named code.png

    # === Server icons ===
    server_dir = os.path.join(ROOT, 'resources', 'server')
    for size in [192, 512]:
        resized = img.resize((size, size), Image.LANCZOS)
        resized.save(os.path.join(server_dir, f'code-{size}.png'))
        print(f'Generated resources/server/code-{size}.png')

print('All icons generated successfully!')

if __name__ == '__main__':
    convert_icons()
