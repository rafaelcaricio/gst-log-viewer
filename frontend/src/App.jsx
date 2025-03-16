import React, { useState, useEffect } from 'react';
import { Toaster } from './components/ui/toaster';
import { useToast } from './components/ui/use-toast';
import FileUpload from './components/FileUpload';
import LogViewer from './components/LogViewer';
import FilterPanel from './components/FilterPanel';
import Navbar from './components/Navbar';

function App() {
  const [sessionId, setSessionId] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [filterOptions, setFilterOptions] = useState(null);
  const [filters, setFilters] = useState({
    level: null,
    categories: [],
    message_regex: null,
    pid: null,
    thread: null,
    object: null,
    function_regex: null,
  });
  const [pagination, setPagination] = useState({
    page: 1,
    per_page: 100,
  });
  
  const { toast } = useToast();
  
  const handleUploadSuccess = async (uploadedSessionId) => {
    console.log(`File uploaded successfully with session ID: ${uploadedSessionId}`);
    setSessionId(uploadedSessionId);
    setIsLoading(true);
    
    // Start polling for filter options
    pollForFilterOptions(uploadedSessionId, 0);
  };
  
  const pollForFilterOptions = async (uploadedSessionId, attempt) => {
    const maxAttempts = 10;
    const delayMs = 1000;
    
    if (attempt >= maxAttempts) {
      console.error(`Failed to fetch filter options after ${maxAttempts} attempts`);
      setIsLoading(false);
      toast({
        title: 'Error',
        description: `Failed to fetch filter options after ${maxAttempts} attempts. The log file may be too large or in an incorrect format.`,
        variant: 'destructive',
      });
      return;
    }
    
    console.log(`Fetching filter options, attempt ${attempt + 1}/${maxAttempts}`);
    
    try {
      const url = `/api/filter-options?session_id=${uploadedSessionId}`;
      console.log(`Making request to: ${url}`);
      
      const response = await fetch(url);
      console.log(`Response status: ${response.status}`);
      
      if (response.status === 404) {
        // Session not found yet, the file might still be processing
        console.log('Session not found yet, waiting before retrying...');
        setTimeout(() => pollForFilterOptions(uploadedSessionId, attempt + 1), delayMs);
        return;
      }
      
      if (!response.ok) {
        // Try to get more detailed error information
        let errorMessage = 'Failed to fetch filter options';
        try {
          const errorJson = await response.json();
          if (errorJson && errorJson.error) {
            errorMessage = errorJson.error;
          }
        } catch (e) {
          console.error('Error parsing error response:', e);
        }
        
        throw new Error(errorMessage);
      }
      
      const options = await response.json();
      console.log('Filter options loaded:', options);
      
      setFilterOptions(options);
      setIsLoading(false);
      
      toast({
        title: 'Log file uploaded successfully',
        description: `Loaded ${options.categories.length} categories with ${options.levels.length} log levels`,
      });
    } catch (error) {
      console.error('Error fetching filter options:', error);
      
      // If there was an error that's not a 404, still retry
      setTimeout(() => pollForFilterOptions(uploadedSessionId, attempt + 1), delayMs);
    }
  };
  
  const handleFilterChange = (newFilters) => {
    setFilters(newFilters);
    setPagination({ ...pagination, page: 1 }); // Reset to first page on filter change
  };
  
  const handlePageChange = (newPage) => {
    setPagination({ ...pagination, page: newPage });
  };
  
  const handlePerPageChange = (newPerPage) => {
    setPagination({ page: 1, per_page: newPerPage });
  };
  
  const handleRetry = () => {
    if (sessionId) {
      setIsLoading(true);
      pollForFilterOptions(sessionId, 0);
    }
  };
  
  const handleReset = () => {
    setSessionId(null);
    setFilterOptions(null);
    setIsLoading(false);
    setFilters({
      level: null,
      categories: [],
      message_regex: null,
      pid: null,
      thread: null,
      object: null,
      function_regex: null,
    });
    setPagination({
      page: 1,
      per_page: 100,
    });
  };
  
  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar onReset={handleReset} sessionActive={!!sessionId} />
      
      <main className="container mx-auto py-6 px-4">
        {!sessionId ? (
          <FileUpload onUploadSuccess={handleUploadSuccess} />
        ) : (
          <>
            {isLoading && !filterOptions ? (
              <div className="text-center py-20">
                <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary mb-4"></div>
                <h2 className="text-lg font-semibold mb-2">Processing Log File</h2>
                <p className="text-gray-500 mb-4">
                  This may take a moment for large files...
                </p>
              </div>
            ) : !filterOptions ? (
              <div className="text-center py-20">
                <div className="bg-red-100 text-red-800 p-4 rounded-lg mb-6 inline-block">
                  <p className="font-bold">Failed to fetch filter options</p>
                  <p className="text-sm mt-1">The log file may be too large or in an incorrect format.</p>
                </div>
                <div className="flex justify-center space-x-4">
                  <button
                    className="px-4 py-2 bg-primary text-white rounded-md hover:bg-primary/90"
                    onClick={handleRetry}
                  >
                    Retry
                  </button>
                  <button
                    className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
                    onClick={handleReset}
                  >
                    Upload a Different File
                  </button>
                </div>
              </div>
            ) : (
              <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                <div className="lg:col-span-1">
                  <FilterPanel 
                    isLoading={isLoading}
                    filterOptions={filterOptions}
                    filters={filters}
                    onFilterChange={handleFilterChange}
                  />
                </div>
                
                <div className="lg:col-span-3">
                  <LogViewer 
                    sessionId={sessionId}
                    filters={filters}
                    pagination={pagination}
                    onPageChange={handlePageChange}
                    onPerPageChange={handlePerPageChange}
                  />
                </div>
              </div>
            )}
          </>
        )}
      </main>
      
      <Toaster />
    </div>
  );
}

export default App;
