#!/usr/bin/env ruby
# frozen_string_literal: true

# Test inline content feature with Fob bundler (Ruby)
#
# Demonstrates bundling inline JavaScript/TypeScript code without file I/O.

require 'tmpdir'
require 'fileutils'

# Load the native extension
begin
  require 'fob_ruby'
rescue LoadError
  # Try loading from workspace target directory
  target_dir = File.expand_path('../../../../target', __FILE__)
  debug_bundle = File.join(target_dir, 'debug', 'fob_ruby.bundle')
  release_bundle = File.join(target_dir, 'release', 'fob_ruby.bundle')
  debug_so = File.join(target_dir, 'debug', 'fob_ruby.so')
  release_so = File.join(target_dir, 'release', 'fob_ruby.so')

  if File.exist?(debug_bundle)
    require debug_bundle
  elsif File.exist?(release_bundle)
    require release_bundle
  elsif File.exist?(debug_so)
    require debug_so
  elsif File.exist?(release_so)
    require release_so
  else
    puts "❌ Error: fob_ruby module not found!"
    puts "\nTo build the extension:"
    puts "  cd ../../.."
    puts "  cargo build --package fob-ruby"
    exit 1
  end
end

def main
  puts "🚀 Testing inline content with Fob (Ruby)...\n"

  # Create a temporary directory for output (in project dir for security)
  script_dir = File.dirname(File.expand_path(__FILE__))
  temp_dir = File.join(script_dir, '.tmp-inline-test')
  FileUtils.mkdir_p(temp_dir)
  puts "📁 Output directory: #{temp_dir}\n"

  begin
    # Initialize logging
    Fob.init_logging("info")

    # Test 1: Single inline content entry
    puts "Test 1: Single inline content entry"
    bundler1 = Fob::Bundler.new(
      entries: [{
        content: "console.log('Hello from Ruby inline content!');",
        name: "main.js"
      }],
      out_dir: "#{temp_dir}/test1",
      format: :esm
    )
    result1 = bundler1.bundle
    chunk1 = result1[:chunks][0]
    puts "✅ Generated: #{chunk1[:file_name]} (#{chunk1[:size]} bytes)\n"

    # Test 2: Multiple inline content entries
    puts "Test 2: Multiple inline content entries"
    bundler2 = Fob::Bundler.new(
      entries: [
        {
          content: "console.log('Entry 1: Hello from inline!');",
          name: "entry1.js"
        },
        {
          content: "console.log('Entry 2: Another inline file!');",
          name: "entry2.js"
        }
      ],
      out_dir: "#{temp_dir}/test2",
      format: :esm
    )
    result2 = bundler2.bundle
    puts "✅ Chunks generated:"
    result2[:chunks].each do |chunk|
      puts "  - #{chunk[:file_name]} (#{chunk[:size]} bytes)"
    end
    puts

    # Test 3: Mixed inline and file entries
    puts "Test 3: Mixed inline content and file path"
    script_dir = File.dirname(File.expand_path(__FILE__))
    bundler3 = Fob::Bundler.new(
      entries: [
        File.join(script_dir, 'src', 'index.js'),  # File path
        {
          content: "console.log('Plus inline content!');",
          name: "inline.js"
        }
      ],
      out_dir: "#{temp_dir}/test3",
      format: :esm
    )
    result3 = bundler3.bundle
    puts "✅ Chunks generated:"
    result3[:chunks].each do |chunk|
      puts "  - #{chunk[:file_name]} (#{chunk[:size]} bytes)"
    end
    puts

    # Test 4: TypeScript inline content
    puts "Test 4: TypeScript inline content"
    bundler4 = Fob::Bundler.new(
      entries: [{
        content: "const message: string = 'TypeScript works!'; console.log(message);",
        name: "typed.ts",
        loader: "ts"
      }],
      out_dir: "#{temp_dir}/test4",
      format: :esm
    )
    result4 = bundler4.bundle
    chunk4 = result4[:chunks][0]
    puts "✅ Generated: #{chunk4[:file_name]} (#{chunk4[:size]} bytes)\n"

    puts "✅ All tests passed!\n"
    puts "📊 Summary:"
    puts "  Test 1: #{result1[:stats][:total_modules]} modules, #{result1[:stats][:total_size]} bytes"
    puts "  Test 2: #{result2[:stats][:total_modules]} modules, #{result2[:stats][:total_size]} bytes"
    puts "  Test 3: #{result3[:stats][:total_modules]} modules, #{result3[:stats][:total_size]} bytes"
    puts "  Test 4: #{result4[:stats][:total_modules]} modules, #{result4[:stats][:total_size]} bytes"

  rescue Fob::Error => e
    puts "❌ Test failed: #{e.message}"
    exit 1
  rescue StandardError => e
    puts "❌ Unexpected error: #{e.message}"
    puts e.backtrace
    exit 1
  ensure
    # Clean up temp directory
    puts "\n🧹 Cleaning up #{temp_dir}"
    FileUtils.rm_rf(temp_dir)
  end
end

main if __FILE__ == $PROGRAM_NAME
