import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react';

const LogViewer = ({ sessionId, filters, pagination, onPageChange, onPerPageChange }) => {
  const [logEntries, setLogEntries] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [totalEntries, setTotalEntries] = useState(0);
  const [totalPages, setTotalPages] = useState(1);
  
  useEffect(() => {
    if (!sessionId) return;
    
    const fetchLogs = async () => {
      setIsLoading(true);
      
      try {
        // Construct query parameters from filters and pagination
        const queryParams = new URLSearchParams({
          session_id: sessionId,
          page: pagination.page,
          per_page: pagination.per_page,
        });
        
        // Add optional filters
        if (filters.level) queryParams.append('level', filters.level);
        if (filters.category) queryParams.append('category', filters.category);
        if (filters.message_regex) queryParams.append('message_regex', filters.message_regex);
        if (filters.pid) queryParams.append('pid', filters.pid);
        if (filters.thread) queryParams.append('thread', filters.thread);
        if (filters.object) queryParams.append('object', filters.object);
        if (filters.function_regex) queryParams.append('function_regex', filters.function_regex);
        
        const response = await fetch(`/api/logs?${queryParams.toString()}`);
        
        if (!response.ok) {
          throw new Error('Failed to fetch logs');
        }
        
        const data = await response.json();
        setLogEntries(data.entries);
        setTotalEntries(data.total);
        setTotalPages(data.total_pages);
      } catch (error) {
        console.error('Error fetching logs:', error);
      } finally {
        setIsLoading(false);
      }
    };
    
    fetchLogs();
  }, [sessionId, filters, pagination]);
  
  const getLevelClass = (level) => {
    const levelLower = level.toLowerCase();
    if (levelLower.includes('error')) return 'log-entry-error';
    if (levelLower.includes('warn')) return 'log-entry-warning';
    if (levelLower.includes('info')) return 'log-entry-info';
    if (levelLower.includes('debug')) return 'log-entry-debug';
    return '';
  };
  
  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Log Entries</CardTitle>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-500">
            {totalEntries > 0 ? 
              `Showing ${((pagination.page - 1) * pagination.per_page) + 1}-${Math.min(pagination.page * pagination.per_page, totalEntries)} of ${totalEntries} entries` : 
              'No entries found'}
          </span>
          <Select
            value={pagination.per_page.toString()}
            onValueChange={(value) => onPerPageChange(parseInt(value))}
          >
            <SelectTrigger className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="50">50</SelectItem>
              <SelectItem value="100">100</SelectItem>
              <SelectItem value="250">250</SelectItem>
              <SelectItem value="500">500</SelectItem>
              <SelectItem value="1000">1000</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-10">
            <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
          </div>
        ) : logEntries.length === 0 ? (
          <div className="text-center py-10 text-gray-500">
            No log entries found matching the current filters
          </div>
        ) : (
          <>
            <div className="rounded-md border overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="bg-muted">
                      <th className="px-4 py-2 text-left font-medium text-muted-foreground">Timestamp</th>
                      <th className="px-4 py-2 text-left font-medium text-muted-foreground">Level</th>
                      <th className="px-4 py-2 text-left font-medium text-muted-foreground">Category</th>
                      <th className="px-4 py-2 text-left font-medium text-muted-foreground">Message</th>
                    </tr>
                  </thead>
                  <tbody>
                    {logEntries.map((entry, index) => (
                      <tr 
                        key={index} 
                        className={`log-entry ${getLevelClass(entry.level)}`}
                      >
                        <td className="px-4 py-2 align-top font-mono whitespace-nowrap">{entry.ts}</td>
                        <td className="px-4 py-2 align-top whitespace-nowrap">{entry.level}</td>
                        <td className="px-4 py-2 align-top whitespace-nowrap">{entry.category}</td>
                        <td className="px-4 py-2 align-top log-message">
                          <div className="flex flex-col">
                            <span>{entry.message}</span>
                            <span className="text-xs text-gray-500 mt-1">
                              {entry.file}:{entry.line} ({entry.function})
                              {entry.object && <> &lt;{entry.object}&gt;</>}
                            </span>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
            
            {totalPages > 1 && (
              <div className="flex items-center justify-between mt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onPageChange(pagination.page - 1)}
                  disabled={pagination.page === 1}
                >
                  <ChevronLeft className="h-4 w-4 mr-1" />
                  Previous
                </Button>
                
                <div className="text-sm text-gray-500">
                  Page {pagination.page} of {totalPages}
                </div>
                
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onPageChange(pagination.page + 1)}
                  disabled={pagination.page >= totalPages}
                >
                  Next
                  <ChevronRight className="h-4 w-4 ml-1" />
                </Button>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
};

export default LogViewer;
