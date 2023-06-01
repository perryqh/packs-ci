#
# This script generates a YAML file located at tmp/filename_to_digest_map.yml that contains information about the unresolved references in the cache files
# generated by packwerk and packs
#
# The script will exit early if there is any diff, as it's purpose is similar to `rspec --next-failure` –
# to provide feedback about a failing test to write and fix.
#
# Example usage:
#   $ ruby filename_to_digest_map.rb
#
require 'json'
require 'hashdiff'
require 'pathname'
require 'pry'
require 'yaml'
require 'digest'

Dir.chdir('../packs') do
  puts "Running cargo build --release in ../packs"
  system('cargo build --release')
end

command = "time ../packs/target/release/packs generate-cache"
puts "Running: #{command}"
system(command)

output = Pathname.new('tmp/filename_to_digest_map.yml')
filemap = {}
Dir['app/**/*.rb'].each do |f|
  cache_basename = Digest::MD5.hexdigest(f)
  experimental_cache_basename = "#{cache_basename}-experimental"
  cache_path = Pathname.new('tmp/cache/packwerk').join(cache_basename)
  experimental_cache_path = Pathname.new('tmp/cache/packwerk').join(experimental_cache_basename)

  cache = JSON.parse(cache_path.read)['unresolved_references'].sort_by{|h| h['constant_name']}
  experimental_cache = JSON.parse(experimental_cache_path.read)['unresolved_references'].sort_by{|h| h['constant_name']}
  # binding.pry
  diff = Hashdiff.diff(
    cache,
    experimental_cache
  )

  if diff.count == 0
    filemap[f] = [
      cache_path.exist? ? cache_path.to_s : "no cache",
      experimental_cache_path.exist? ? experimental_cache_path.to_s : "no experimental_cache",
      "Noo difference!"
    ]
  else
    filemap[f] = [
      cache_path.exist? ? cache_path.to_s : "no cache",
      experimental_cache_path.exist? ? experimental_cache_path.to_s : "no experimental_cache",
      "cache has #{cache.count} unresolved references",
      "experimental cache has #{experimental_cache.count} unresolved references",
      "diff count is #{diff.count}",
      "cache content: #{cache.inspect}",
      "experimental_cache content: #{experimental_cache.inspect}",
      "diff is #{diff}"
    ]
  end

  break if diff.count > 0
end

output.write(YAML.dump(filemap))
puts "Wrote content to: #{output}"
