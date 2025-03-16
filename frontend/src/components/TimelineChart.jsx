import React, { useState, useEffect, useCallback } from 'react';
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  Brush,
  ReferenceArea
} from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { useResizeDetector } from 'react-resize-detector';

const TimelineChart = ({ sessionId, onTimeRangeChange, filters }) => {
  const [timeData, setTimeData] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [timeInterval, setTimeInterval] = useState('1s');
  const [selecting, setSelecting] = useState(false);
  const [dragStartIndex, setDragStartIndex] = useState(null);
  const [dragEndIndex, setDragEndIndex] = useState(null);
  const [activeSelection, setActiveSelection] = useState(null);
  const { width, ref } = useResizeDetector();

  // Available time intervals for grouping
  const timeIntervals = [
    // Sub-millisecond precision
    { value: '100us', label: '100 Microseconds' },
    { value: '250us', label: '250 Microseconds' },
    { value: '500us', label: '500 Microseconds' },
    // Millisecond precision
    { value: '1ms', label: '1 Millisecond' },
    { value: '5ms', label: '5 Milliseconds' },
    { value: '10ms', label: '10 Milliseconds' },
    { value: '50ms', label: '50 Milliseconds' },
    { value: '100ms', label: '100 Milliseconds' },
    { value: '500ms', label: '500 Milliseconds' },
    // Second precision
    { value: '1s', label: '1 Second' },
    { value: '5s', label: '5 Seconds' },
    { value: '10s', label: '10 Seconds' },
    { value: '30s', label: '30 Seconds' },
    // Minute precision
    { value: '1m', label: '1 Minute' },
    { value: '5m', label: '5 Minutes' },
  ];

  // Fetch timeline data
  const fetchTimelineData = useCallback(async () => {
    if (!sessionId) return;
    
    setIsLoading(true);
    
    try {
      // Construct query parameters from filters
      const queryParams = new URLSearchParams({
        session_id: sessionId,
        interval: timeInterval,
      });
      
      // Add filters if they exist
      if (filters.level) queryParams.append('level', filters.level);
      if (filters.categories && filters.categories.length > 0) {
        filters.categories.forEach(category => {
          queryParams.append('categories', category);
        });
      }
      if (filters.message_regex) queryParams.append('message_regex', filters.message_regex);
      if (filters.pid) queryParams.append('pid', filters.pid);
      if (filters.thread) queryParams.append('thread', filters.thread);
      if (filters.object) queryParams.append('object', filters.object);
      if (filters.function_regex) queryParams.append('function_regex', filters.function_regex);
      
      const url = `/api/timeline?${queryParams.toString()}`;
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error('Failed to fetch timeline data');
      }
      
      const data = await response.json();
      
      // Process the data to format timestamps nicely for display
      const processedData = data.buckets.map(bucket => ({
        timestamp: bucket.timestamp,
        // Format time as human-readable, e.g. "0:05.215"
        displayTime: formatTime(bucket.timestamp, timeInterval.endsWith('us')),
        count: bucket.count
      }));
      
      setTimeData(processedData);
    } catch (error) {
      console.error('Error fetching timeline data:', error);
    } finally {
      setIsLoading(false);
    }
  }, [sessionId, timeInterval, filters]);

  useEffect(() => {
    fetchTimelineData();
  }, [fetchTimelineData]);
  
  // Format time in a more readable way
  const formatTime = (timestampMs, usesMicroseconds = false) => {
    // Set the divisor based on time precision
    const divisor = usesMicroseconds ? 1_000_000 : 1000;
    
    // Convert to seconds
    const totalSeconds = timestampMs / divisor;
    
    // Extract minutes, seconds and milliseconds
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = Math.floor(totalSeconds % 60);
    const milliseconds = Math.floor((totalSeconds % 1) * 1000);
    
    // Format string as M:SS.mmm
    return `${minutes}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(3, '0')}`;
  };
  
  // Custom tooltip that displays time and count
  const CustomTooltip = ({ active, payload }) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white p-2 border rounded shadow-md text-xs">
          <p className="font-semibold">{payload[0].payload.displayTime}</p>
          <p>
            <span className="font-medium">{payload[0].value}</span> log entries
          </p>
        </div>
      );
    }
    return null;
  };
  
  // Handle time interval change
  const handleIntervalChange = (value) => {
    setTimeInterval(value);
    // Reset selection and clear filter
    clearSelection();
  };
  
  // Find the index of a chart data point by display time
  const findDataIndexByLabel = (label) => {
    if (!label || !timeData.length) return -1;
    return timeData.findIndex(d => d.displayTime === label);
  };
  
  // Apply time range selection to filters
  const applyTimeRangeSelection = (startIdx, endIdx) => {
    if (startIdx === null || endIdx === null || !timeData.length) {
      return;
    }
    
    // Get min/max indices
    const minIdx = Math.min(startIdx, endIdx);
    const maxIdx = Math.max(startIdx, endIdx);
    
    // Validate indices
    if (minIdx < 0 || minIdx >= timeData.length || maxIdx < 0 || maxIdx >= timeData.length) {
      console.error("Invalid indices:", minIdx, maxIdx);
      return;
    }
    
    // Get timestamps from indices
    const startTs = timeData[minIdx].timestamp;
    const endTs = timeData[maxIdx].timestamp;
    
    // For a single point selection (click), add a small range around it
    // to ensure we capture the relevant logs
    const isSinglePointSelection = minIdx === maxIdx;
    
    // Create selection object
    setActiveSelection({
      startIndex: minIdx,
      endIndex: maxIdx,
      start: startTs,
      end: endTs,
      isSinglePoint: isSinglePointSelection
    });
    
    // Create time range filter object
    const usesMicroseconds = timeInterval.endsWith('us');
    
    // If it's a single-point selection, create a small window around it
    let rangeData;
    if (isSinglePointSelection) {
      // Determine an appropriate window size based on time interval
      const intervalUnit = timeInterval.slice(-2);
      let windowSize = 1;  // Default size
      
      if (intervalUnit === 'us') {
        windowSize = 1000; // 1000 microseconds
      } else if (intervalUnit === 'ms') {
        windowSize = 10;   // 10 milliseconds
      } else if (intervalUnit === 's') {
        windowSize = 1;    // 1 second
      } else if (intervalUnit === 'm') {
        windowSize = 1;    // 1 minute
      }
      
      // Create a range around the selected point
      rangeData = {
        min: Math.max(0, startTs - windowSize),
        max: startTs + windowSize,
        useMicroseconds: usesMicroseconds
      };
    } else {
      // Normal range selection
      rangeData = {
        min: startTs,
        max: endTs,
        useMicroseconds: usesMicroseconds
      };
    }
    
    console.log("Setting time range:", rangeData);
    
    // Notify parent component
    if (onTimeRangeChange) {
      onTimeRangeChange(rangeData);
    }
  };
  
  // Mouse event handlers for selection
  const handleMouseDown = (e) => {
    if (!e || !e.activeLabel) return;
    
    const index = findDataIndexByLabel(e.activeLabel);
    if (index === -1) return;
    
    setDragStartIndex(index);
    setDragEndIndex(index);
    setSelecting(true);
  };
  
  const handleMouseMove = (e) => {
    if (!selecting || !e || !e.activeLabel) return;
    
    const index = findDataIndexByLabel(e.activeLabel);
    if (index !== -1) {
      setDragEndIndex(index);
    }
  };
  
  const handleMouseUp = () => {
    if (!selecting) return;
    
    setSelecting(false);
    
    // Apply the selection regardless of whether it's a click or a drag
    applyTimeRangeSelection(dragStartIndex, dragEndIndex);
  };

  const handleBrushChange = (brushProps) => {
    // Only act when the brush finishes changing (on mouse up)
    if (!brushProps) return;
    
    const { startIndex, endIndex } = brushProps;
    
    // Skip empty selections
    if (startIndex === undefined || endIndex === undefined) {
      return;
    }
    
    // Validate indices
    if (startIndex >= 0 && startIndex < timeData.length &&
        endIndex >= 0 && endIndex < timeData.length) {
      
      applyTimeRangeSelection(startIndex, endIndex);
    }
  };
  
  // Clear selection
  const clearSelection = () => {
    setDragStartIndex(null);
    setDragEndIndex(null);
    setActiveSelection(null);
    
    if (onTimeRangeChange) {
      onTimeRangeChange(null);
    }
  };

  return (
    <Card className="w-full mb-4">
      <CardHeader className="px-4 py-2 flex flex-row items-center justify-between">
        <CardTitle className="text-base">Log Timeline</CardTitle>
        <div className="flex items-center space-x-2">
          <span className="text-xs text-gray-500">Group by:</span>
          <Select
            value={timeInterval}
            onValueChange={handleIntervalChange}
          >
            <SelectTrigger className="h-8 text-xs w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="divider1" disabled className="py-1 text-xs text-gray-500 border-b">
                Microseconds
              </SelectItem>
              {timeIntervals.slice(0, 3).map((interval) => (
                <SelectItem key={interval.value} value={interval.value}>
                  {interval.label}
                </SelectItem>
              ))}
              
              <SelectItem value="divider2" disabled className="py-1 text-xs text-gray-500 border-b border-t">
                Milliseconds
              </SelectItem>
              {timeIntervals.slice(3, 9).map((interval) => (
                <SelectItem key={interval.value} value={interval.value}>
                  {interval.label}
                </SelectItem>
              ))}
              
              <SelectItem value="divider3" disabled className="py-1 text-xs text-gray-500 border-b border-t">
                Seconds
              </SelectItem>
              {timeIntervals.slice(9, 13).map((interval) => (
                <SelectItem key={interval.value} value={interval.value}>
                  {interval.label}
                </SelectItem>
              ))}
              
              <SelectItem value="divider4" disabled className="py-1 text-xs text-gray-500 border-b border-t">
                Minutes
              </SelectItem>
              {timeIntervals.slice(13).map((interval) => (
                <SelectItem key={interval.value} value={interval.value}>
                  {interval.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </CardHeader>
      <CardContent className="p-2">
        {isLoading ? (
          <div className="flex items-center justify-center h-32">
            <div className="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
          </div>
        ) : timeData.length === 0 ? (
          <div className="text-center py-4 text-gray-500 text-sm">
            No timeline data available
          </div>
        ) : (
          <div ref={ref} className="w-full">
            <ResponsiveContainer width="100%" height={160}>
              <BarChart
                data={timeData}
                margin={{ top: 0, right: 0, left: 0, bottom: 20 }}
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
                onMouseLeave={handleMouseUp}
              >
                <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#eee" />
                <XAxis 
                  dataKey="displayTime"
                  angle={-45}
                  textAnchor="end"
                  tick={{ fontSize: 10 }}
                  interval="preserveStartEnd"
                  minTickGap={40}
                  tickFormatter={(val, idx) => {
                    const visibleTicksCount = Math.floor(width / 80);
                    if (timeData.length <= visibleTicksCount) return val;
                    
                    // Show first, last, and some evenly distributed ticks
                    if (idx === 0 || idx === timeData.length - 1) return val;
                    if (idx % Math.ceil(timeData.length / visibleTicksCount) === 0) return val;
                    return '';
                  }}
                />
                <YAxis tick={{ fontSize: 10 }} width={25} />
                <Tooltip content={<CustomTooltip />} />
                <Brush
                  dataKey="displayTime"
                  height={20}
                  stroke="#8884d8"
                  onChange={handleBrushChange}
                  travellerWidth={8}
                />
                <Bar 
                  dataKey="count" 
                  fill="#3b82f6" 
                  radius={[2, 2, 0, 0]}
                  animationDuration={300}
                />
                {/* Show selection during dragging */}
                {selecting && dragStartIndex !== null && dragEndIndex !== null && timeData.length > 0 && (
                  <ReferenceArea
                    x1={timeData[Math.min(dragStartIndex, dragEndIndex)]?.displayTime}
                    x2={timeData[Math.max(dragStartIndex, dragEndIndex)]?.displayTime}
                    strokeOpacity={0.3}
                    fill="#3b82f6"
                    fillOpacity={0.3}
                  />
                )}
                
                {/* Show active selection */}
                {!selecting && activeSelection && (
                  <ReferenceArea
                    x1={timeData[activeSelection.startIndex]?.displayTime}
                    x2={timeData[activeSelection.endIndex]?.displayTime}
                    strokeOpacity={0.3}
                    fill="#3b82f6"
                    fillOpacity={0.3}
                  />
                )}
              </BarChart>
            </ResponsiveContainer>
            
            {/* Selection info and clear button */}
            {activeSelection && (
              <div className="flex justify-between items-center mt-1 px-1">
                <div className="text-xs text-gray-700">
                  <span className="font-medium">Selected:</span> {activeSelection.isSinglePoint 
                    ? `Time point ${formatTime(activeSelection.start, timeInterval.endsWith('us'))}`
                    : `${formatTime(activeSelection.start, timeInterval.endsWith('us'))} to ${formatTime(activeSelection.end, timeInterval.endsWith('us'))}`
                  }
                </div>
                <button
                  onClick={clearSelection}
                  className="px-2 py-0.5 text-xs bg-gray-200 hover:bg-gray-300 text-gray-700 rounded"
                >
                  Clear
                </button>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};

export default TimelineChart;