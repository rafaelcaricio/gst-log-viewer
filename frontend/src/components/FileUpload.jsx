import React, { useState, useRef } from 'react';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Upload, Info } from 'lucide-react';

const FileUpload = ({ onUploadSuccess }) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const [fileName, setFileName] = useState('');
  const fileInputRef = useRef(null);

  const handleDragOver = (e) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const handleDrop = (e) => {
    e.preventDefault();
    setIsDragging(false);
    
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      setFileName(file.name);
      uploadFile(file);
    }
  };

  const handleFileChange = (e) => {
    if (e.target.files && e.target.files.length > 0) {
      const file = e.target.files[0];
      setFileName(file.name);
      uploadFile(file);
    }
  };

  const handleButtonClick = () => {
    fileInputRef.current.click();
  };

  const uploadFile = async (file) => {
    setIsUploading(true);
    
    try {
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Upload failed');
      }

      const data = await response.json();
      onUploadSuccess(data.session_id);
    } catch (error) {
      console.error('Error uploading file:', error);
      setIsUploading(false);
    }
  };

  return (
    <div className="space-y-6">
      <Card className="w-full max-w-2xl mx-auto">
        <CardHeader>
          <CardTitle>Upload GStreamer Log File</CardTitle>
          <CardDescription>
            Upload a GStreamer log file to analyze and visualize its contents
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div
            className={`border-2 border-dashed rounded-lg p-12 text-center ${
              isDragging ? 'border-primary bg-primary/5' : 'border-gray-300'
            }`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <input
              type="file"
              ref={fileInputRef}
              className="hidden"
              accept=".log,.txt"
              onChange={handleFileChange}
            />
            <Upload className="h-12 w-12 mx-auto mb-4 text-gray-400" />
            <p className="text-lg font-medium mb-1">
              Drag and drop your log file here
            </p>
            <p className="text-sm text-gray-500 mb-4">
              or click to browse your files
            </p>
            <Button onClick={handleButtonClick} disabled={isUploading}>
              {isUploading ? 'Uploading...' : 'Select Log File'}
            </Button>
            {fileName && (
              <p className="mt-4 text-sm text-gray-700">Selected: {fileName}</p>
            )}
          </div>
        </CardContent>
        <CardFooter className="text-sm text-gray-500">
          Supported formats: .log, .txt
        </CardFooter>
      </Card>

      <Card className="w-full max-w-2xl mx-auto">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Info className="h-5 w-5" />
            How to Capture GStreamer Logs
          </CardTitle>
          <CardDescription>
            Follow these instructions to generate log files for analysis
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <h3 className="text-base font-semibold mb-2">1. Set debug environment variables</h3>
              <p className="text-sm text-gray-700 mb-2">
                To capture GStreamer logs, set these environment variables before running your application:
              </p>
              <div className="bg-gray-100 p-3 rounded-md font-mono text-sm overflow-x-auto">
                <code>export GST_DEBUG=2,*:3</code><br />
                <code>export GST_DEBUG_FILE=$PWD/gstreamer.log</code>
              </div>
              <p className="text-xs text-gray-500 mt-2">
                This will create a log file named 'gstreamer.log' in your current directory with level 3 verbosity.
              </p>
            </div>

            <div>
              <h3 className="text-base font-semibold mb-2">2. Adjust verbosity levels</h3>
              <p className="text-sm text-gray-700 mb-2">
                Control log verbosity with the <code className="bg-gray-100 px-1 rounded">GST_DEBUG</code> variable:
              </p>
              <ul className="list-disc list-inside text-sm text-gray-700 space-y-1">
                <li><code className="bg-gray-100 px-1 rounded">0</code> - No debug information</li>
                <li><code className="bg-gray-100 px-1 rounded">1</code> - Error messages</li>
                <li><code className="bg-gray-100 px-1 rounded">2</code> - Warnings</li>
                <li><code className="bg-gray-100 px-1 rounded">3</code> - Informational messages</li>
                <li><code className="bg-gray-100 px-1 rounded">4</code> - Debug messages</li>
                <li><code className="bg-gray-100 px-1 rounded">5</code> - Log messages</li>
                <li><code className="bg-gray-100 px-1 rounded">6+</code> - Trace messages</li>
              </ul>
            </div>

            <div>
              <h3 className="text-base font-semibold mb-2">3. Target specific categories</h3>
              <p className="text-sm text-gray-700 mb-2">
                Get logs from specific GStreamer elements or categories:
              </p>
              <div className="bg-gray-100 p-3 rounded-md font-mono text-sm overflow-x-auto">
                <code>export GST_DEBUG=videotestsrc:6,audio*:4</code>
              </div>
              <p className="text-xs text-gray-500 mt-2">
                This example sets 'videotestsrc' to level 6 and all audio-related categories to level 4.
              </p>
            </div>

            <div>
              <h3 className="text-base font-semibold mb-2">4. Run your application</h3>
              <p className="text-sm text-gray-700">
                After setting the environment variables, run your GStreamer application normally. When finished, upload 
                the generated log file using the form above for detailed analysis.
              </p>
            </div>
          </div>
        </CardContent>
        <CardFooter className="text-sm text-gray-500">
          For more detailed information, visit the <a href="https://gstreamer.freedesktop.org/documentation/gstreamer/running.html?gi-language=c#running-and-debugging-gstreamer-applications" target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">GStreamer Debugging Guide</a>.
        </CardFooter>
      </Card>
    </div>
  );
};

export default FileUpload;
