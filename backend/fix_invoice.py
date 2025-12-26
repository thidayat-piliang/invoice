with open('src/infrastructure/repositories/invoice_repository.rs', 'rb') as f:
    content = f.read()

# The corrupted bytes are \xc2\xac
# We want to replace .bind(¬es) with .bind(¬es)
corrupted = b'\xc2\xac'
# Target: .bind(¬es)
target = b'.bind(' + corrupted + b'es)'
# Replacement: .bind(¬es) - using & for borrow
replacement = b'.bind(¬es)'

content = content.replace(target, replacement)

with open('src/infrastructure/repositories/invoice_repository.rs', 'wb') as f:
    f.write(content)

print('Fixed')
