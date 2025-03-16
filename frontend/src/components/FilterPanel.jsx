import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Search, X, Check, ChevronDown } from 'lucide-react';

const FilterPanel = ({ isLoading, filterOptions, filters, onFilterChange }) => {
  const [categoriesOpen, setCategoriesOpen] = useState(false);

  const handleFilter = (key, value) => {
    const newFilters = { ...filters, [key]: value };
    onFilterChange(newFilters);
  };

  const handleCategoryToggle = (category) => {
    let newCategories;

    if (filters.categories.includes(category)) {
      // Remove the category if it's already selected
      newCategories = filters.categories.filter(c => c !== category);
    } else {
      // Add the category if it's not selected
      newCategories = [...filters.categories, category];
    }

    handleFilter('categories', newCategories);
  };

  const handleClearFilters = () => {
    // Make sure to use an empty array for categories
    onFilterChange({
      level: null,
      categories: [],
      message_regex: null,
      pid: null,
      thread: null,
      object: null,
      function_regex: null,
    });
  };

  const isCategorySelected = (category) => {
    return filters.categories.includes(category);
  };

  const getCategoriesDisplayText = () => {
    if (!filters.categories || filters.categories.length === 0) {
      return 'All categories';
    }
    if (filters.categories.length === 1) {
      return filters.categories[0];
    }
    return `${filters.categories.length} categories selected`;
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
          <label className="text-sm font-medium">Categories</label>
          <div className="relative">
            <div 
              className="flex items-center justify-between border rounded-md px-3 py-2 text-sm cursor-pointer hover:border-gray-400"
              onClick={() => setCategoriesOpen(!categoriesOpen)}
            >
              <span>{getCategoriesDisplayText()}</span>
              <ChevronDown className="h-4 w-4" />
            </div>
            
            {categoriesOpen && (
              <div className="absolute z-50 w-full mt-1 bg-white border rounded-md shadow-lg max-h-60 overflow-auto">
                {filterOptions.categories.map((category) => (
                  <div key={category}
                    className={`px-3 py-2 cursor-pointer flex items-center justify-between hover:bg-gray-100 ${isCategorySelected(category) ? 'bg-blue-50' : ''}`}
                    onClick={() => handleCategoryToggle(category)}
                  >
                    <span>{category}</span>
                    {isCategorySelected(category) && <Check className="h-4 w-4 text-blue-600" />}
                  </div>
                ))}
                {filters.categories.length > 0 && (
                  <div className="border-t px-3 py-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="w-full justify-start text-gray-600"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleFilter('categories', []);
                        setCategoriesOpen(false);
                      }}
                    >
                      Clear selection
                    </Button>
                  </div>
                )}
              </div>
            )}
          </div>
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
