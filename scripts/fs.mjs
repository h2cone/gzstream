import http from 'http';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Get the current file name and directory name
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Define the directory where assets are stored
const ASSETS_DIR = path.join(__dirname, '../assets');

// Create an HTTP server
const server = http.createServer((req, res) => {
    // Parse the request URL
    const url = new URL(req.url, `http://${req.headers.host}`);

    // Handle the /download endpoint
    if (url.pathname === '/download') {
        // Get the 'key' query parameter
        const key = url.searchParams.get('key');

        // If 'key' is not provided, return a 400 error
        if (!key) {
            res.writeHead(400, { 'Content-Type': 'text/plain' });
            return res.end('key is required');
        }

        // Normalize the key to prevent directory traversal attacks
        const normalizedKey = path.normalize(key);

        // Construct the file path
        const filepath = path.join(ASSETS_DIR, normalizedKey);

        // Prevent directory traversal attacks
        if (!filepath.startsWith(ASSETS_DIR)) {
            res.writeHead(403, { 'Content-Type': 'text/plain' });
            return res.end('Access denied');
        }

        // Check if the file exists
        fs.access(filepath, fs.constants.F_OK, (err) => {
            if (err) {
                res.writeHead(404, { 'Content-Type': 'text/plain' });
                return res.end('File not found');
            }

            // Set headers for file download
            res.writeHead(200, {
                'Content-Type': 'application/octet-stream',
                'Content-Disposition': `attachment; filename=${path.basename(filepath)}`
            });

            // Create a read stream and pipe it to the response
            const fileStream = fs.createReadStream(filepath);
            fileStream.pipe(res);

            // Handle file stream errors
            fileStream.on('error', (err) => {
                console.error('File stream error:', err);
                res.writeHead(500, { 'Content-Type': 'text/plain' });
                res.end('Error reading file');
            });
        });
    } else {
        // Return a 404 error for any other endpoints
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('Not found');
    }
});

// Start the server on the specified port
const PORT = process.env.PORT || 3000;
server.listen(PORT, () => {
    console.log(`Server is running on port ${PORT}`);
});