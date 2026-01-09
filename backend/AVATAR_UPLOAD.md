# Avatar Upload Feature

## Overview
The forum backend now supports avatar uploads for users. This feature allows authenticated users to upload profile pictures that will be displayed alongside their posts and profile.

## API Endpoint

### Upload Avatar
**Endpoint:** `POST /users/avatar`

**Authentication:** Required (JWT token)

**Content-Type:** `multipart/form-data`

**Request Body:**
- `avatar`: Image file (JPEG, JPG, PNG, GIF, or WebP format)
- Maximum file size: 5MB (configurable via `MAX_FILE_SIZE` environment variable)

**Response:**
```json
{
  "success": true,
  "message": "Avatar uploaded successfully",
  "avatar_url": "http://localhost:8000/uploads/avatar_<user_id>_<uuid>.jpg",
  "filename": "avatar_<user_id>_<uuid>.jpg"
}
```

**Error Responses:**
- `400 Bad Request`: Invalid file type, file too large, or no file provided
- `401 Unauthorized`: User not authenticated
- `500 Internal Server Error`: Server error during file processing

## Configuration

### Environment Variables
Add these to your `.env` file:

```bash
# File upload configuration
UPLOAD_DIR=./uploads
MAX_FILE_SIZE=5242880  # 5MB in bytes
```

### Default Values
- `UPLOAD_DIR`: `./uploads` (relative to application root)
- `MAX_FILE_SIZE`: `5242880` (5MB)
- `allowed_image_types`: `["image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp"]`

## File Storage

### Location
Uploaded avatars are stored in the directory specified by `UPLOAD_DIR` environment variable (default: `./uploads`).

### Naming Convention
Avatars are named using the pattern: `avatar_<user_id>_<uuid>.<extension>`
- `user_id`: The UUID of the user
- `uuid`: Random UUID to ensure uniqueness
- `extension`: File extension based on content type (jpg, png, gif, webp)

### Old Avatar Cleanup
When a user uploads a new avatar:
1. The old avatar file is automatically deleted (if it exists and is not the default avatar)
2. The default avatar (`default.png`) is never deleted
3. The database is updated with the new avatar filename

## Static File Serving
Avatar files are served statically at: `GET /uploads/<filename>`

The server automatically serves files from the `UPLOAD_DIR` directory.

## Database Schema
The `forum.users` table already has an `avatar` column:
```sql
avatar varchar(100) DEFAULT 'default.png'
```

## Usage Example

### Using curl:
```bash
curl -X POST http://localhost:8000/users/avatar \
  -H "Authorization: Bearer <jwt_token>" \
  -F "avatar=@/path/to/your/image.jpg"
```

### Using JavaScript (fetch):
```javascript
const formData = new FormData();
formData.append('avatar', imageFile);

const response = await fetch('http://localhost:8000/users/avatar', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${jwtToken}`
  },
  body: formData
});

const result = await response.json();
console.log(result.avatar_url); // URL to the uploaded avatar
```

## Security Considerations

1. **File Type Validation**: Only allowed image types are accepted
2. **File Size Limits**: Configurable maximum file size prevents abuse
3. **Unique Filenames**: UUID-based filenames prevent filename collisions
4. **Authentication Required**: Only authenticated users can upload avatars
5. **Old File Cleanup**: Old avatars are automatically deleted to save disk space

## Testing

To test the avatar upload feature:

1. Start the server:
   ```bash
   cargo run
   ```

2. Authenticate a user and obtain a JWT token

3. Upload an avatar using the API endpoint

4. Verify the avatar is accessible at the returned URL

5. Check the database to confirm the avatar field is updated

