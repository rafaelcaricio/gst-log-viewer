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
    category: null,
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
    setSessionId(uploadedSessionId);
    setIsLoading(true);
    
    try {
      const response = await fetch(`/api/filter-options?session_id=${uploadedSessionId}`);
      if (!response.ok) {
        throw new Error('Failed to fetch filter options');
      }
      
      const options = await response.json();
      setFilterOptions(options);
      setIsLoading(false);
      
      toast({
        title: 'Log file uploaded successfully',
        description: `Loaded ${options.categories.length} categories of log entries`,
      });
    } catch (error) {
      setIsLoading(false);
      toast({
        title: 'Error',
        description: error.message,
        variant: 'destructive',
      });
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
  
  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />
      
      <main className="container mx-auto py-6 px-4">
        {!sessionId ? (
          <FileUpload onUploadSuccess={handleUploadSuccess} />
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
      </main>
      
      <Toaster />
    </div>
  );
}

export default App;
