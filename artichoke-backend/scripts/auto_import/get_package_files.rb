# frozen_string_literal: true

# The purpose of this script is to open a fresh interpreter, pull the constants,
# require a library and figure out what constants were added.
BASE = ARGV[0]
PACKAGE = ARGV[1]
puts BASE
$LOAD_PATH.unshift(BASE)
require PACKAGE
puts $LOADED_FEATURES
raise 'abort'
lib_sources = $LOADED_FEATURES.select { |f| f.include?(BASE) }
package_sources = lib_sources.select { |f| f =~ /#{PACKAGE}/ }
puts package_sources.sort
