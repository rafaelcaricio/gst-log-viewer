import React from 'react';
import { Button } from './ui/button';

const Navbar = ({ onReset, sessionActive }) => {
  return (
    <header className="bg-primary">
      <div className="container mx-auto px-4 py-3 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white flex items-center">
          <svg 
            viewBox="0 0 256 256" 
            className="w-8 h-8 mr-2"
            fill="currentColor"
          >
            <path d="M189.7,142.4l-46-46c-3.9-3.9-10.2-3.9-14.1,0l-46,46c-3.9,3.9-3.9,10.2,0,14.1c3.9,3.9,10.2,3.9,14.1,0l39-39l39,39  c2,2,4.5,2.9,7.1,2.9s5.1-1,7.1-2.9C193.6,152.7,193.6,146.3,189.7,142.4z" />
            <path d="M128,18C67.9,18,19.3,66.6,19.3,126.7s48.6,108.7,108.7,108.7s108.7-48.6,108.7-108.7S188.1,18,128,18z M128,216  c-49.2,0-89.3-40.1-89.3-89.3S78.8,37.3,128,37.3s89.3,40.1,89.3,89.3S177.2,216,128,216z" />
          </svg>
          GStreamer Log Viewer
        </h1>
        
        {sessionActive && (
          <Button 
            variant="outline" 
            className="bg-white hover:bg-gray-100 text-primary"
            onClick={onReset}
          >
            Upload New File
          </Button>
        )}
      </div>
    </header>
  );
};

export default Navbar;
