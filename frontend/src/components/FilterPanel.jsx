import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Search, X } from 'lucide-react';

const FilterPanel = ({ isLoading, filterOptions, filters, onFilterChange }) => {
  const handleFilter = (key, value) => {
    const newFilters = { ...filters, [key]: value };
    onFilterChange(newFilters);
  };

  const handleClearFilters = () => {
    onFilterChange({
      level: null,
      category: null,
      message_regex: null,
      pid: null,
      thread: null,
      object: null,
      function_regex: null,
    });
  };

  if (isLoading || !filterOptions) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Filters</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-10">
            <div className="animate-pulse text-gray-400">Loading filters...</div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex justify-between items-center">
          <span>Filters</span>
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={handleClearFilters}
            title="Clear all filters"
          >
            <X className="h-4 w-4" />
          </Button>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <label className="text-sm font-medium">Log Level</label>
          <Select 
            value={filters.level || 'all-levels'} 
            onValueChange={(value) => handleFilter('level', value === 'all-levels' ? null : value)}
          >
            <SelectTrigger>
              <SelectValue placeholder="All levels" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all-levels">All levels</SelectItem>
              {filterOptions.levels.map((level) => (
                <SelectItem key={level} value={level}>
                  {level}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Category</label>
          <Select 
            value={filters.category || 'all-categories'} 
            onValueChange={(value) => handleFilter('category', value === 'all-categories' ? null : value)}
          >
            <SelectTrigger>
              <SelectValue placeholder="All categories" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all-categories">All categories</SelectItem>
              {filterOptions.categories.map((category) => (
                <SelectItem key={category} value={category}>
                  {category}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Message (Regex)</label>
          <div className="flex">
            <Input
              value={filters.message_regex || ''}
              onChange={(e) => handleFilter('message_regex', e.target.value || null)}
              placeholder="Search in messages..."
            />
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">PID</label>
          <Select 
            value={filters.pid ? String(filters.pid) : 'all-pids'} 
            onValueChange={(value) => handleFilter('pid', value === 'all-pids' ? null : parseInt(value))}
          >
            <SelectTrigger>
              <SelectValue placeholder="All PIDs" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all-pids">All PIDs</SelectItem>
              {filterOptions.pids.map((pid) => (
                <SelectItem key={pid} value={String(pid)}>
                  {pid}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Thread</label>
          <Select 
            value={filters.thread || 'all-threads'} 
            onValueChange={(value) => handleFilter('thread', value === 'all-threads' ? null : value)}
          >
            <SelectTrigger>
              <SelectValue placeholder="All threads" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all-threads">All threads</SelectItem>
              {filterOptions.threads.map((thread) => (
                <SelectItem key={thread} value={thread}>
                  {thread}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Object</label>
          <Select 
            value={filters.object || 'all-objects'} 
            onValueChange={(value) => handleFilter('object', value === 'all-objects' ? null : value)}
          >
            <SelectTrigger>
              <SelectValue placeholder="All objects" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all-objects">All objects</SelectItem>
              {filterOptions.objects.map((object) => (
                <SelectItem key={object} value={object}>
                  {object}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium">Function (Regex)</label>
          <Input
            value={filters.function_regex || ''}
            onChange={(e) => handleFilter('function_regex', e.target.value || null)}
            placeholder="Search in functions..."
          />
        </div>
      </CardContent>
    </Card>
  );
};

export default FilterPanel;
