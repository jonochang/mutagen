require "cgi"

module Mutagen
  module Reporter
    class Html
      def report(results, output_path: "mutagen_report.html")
        killed = results.count { |r| r.status == "killed" }
        survived = results.count { |r| r.status == "survived" }
        timeout = results.count { |r| r.status == "timeout" }
        errored = results.count { |r| r.status == "error" }
        scoreable = killed + survived
        score = scoreable > 0 ? (killed.to_f / scoreable * 100).round(1) : 0.0

        by_file = results.group_by { |r| r.mutation[:file] }

        html = build_html(results, by_file, killed, survived, timeout, errored, score)
        File.write(output_path, html)
        output_path
      end

      private

      def build_html(results, by_file, killed, survived, timeout, errored, score)
        <<~HTML
          <!DOCTYPE html>
          <html lang="en">
          <head>
            <meta charset="utf-8">
            <title>Mutagen Report</title>
            <style>#{css}</style>
          </head>
          <body>
            <div class="container">
              <h1>Mutagen Mutation Testing Report</h1>
              #{summary_section(killed, survived, timeout, errored, score)}
              #{file_table(by_file)}
              #{file_details(by_file)}
            </div>
          </body>
          </html>
        HTML
      end

      def summary_section(killed, survived, timeout, errored, score)
        total = killed + survived + timeout + errored
        score_class = if score >= 80
                        "score-high"
                      elsif score >= 60
                        "score-medium"
                      else
                        "score-low"
                      end
        <<~HTML
          <div class="summary">
            <div class="score #{score_class}">#{score}%</div>
            <div class="stats">
              <span class="stat killed">#{killed} killed</span>
              <span class="stat survived">#{survived} survived</span>
              <span class="stat timeout">#{timeout} timed out</span>
              <span class="stat errored">#{errored} errored</span>
              <span class="stat total">#{total} total</span>
            </div>
          </div>
        HTML
      end

      def file_table(by_file)
        rows = by_file.sort_by { |f, _| f }.map do |file, file_results|
          fk = file_results.count { |r| r.status == "killed" }
          fs = file_results.count { |r| r.status == "survived" }
          ft = fk + fs
          fscore = ft > 0 ? (fk.to_f / ft * 100).round(1) : 0.0
          anchor = file_anchor(file)
          score_class = fscore >= 80 ? "score-high" : (fscore >= 60 ? "score-medium" : "score-low")
          <<~ROW
            <tr>
              <td><a href="##{anchor}">#{h(file)}</a></td>
              <td>#{ft}</td>
              <td>#{fk}</td>
              <td>#{fs}</td>
              <td class="#{score_class}">#{fscore}%</td>
            </tr>
          ROW
        end.join

        <<~HTML
          <h2>File Summary</h2>
          <table class="file-table">
            <thead>
              <tr><th>File</th><th>Mutations</th><th>Killed</th><th>Survived</th><th>Score</th></tr>
            </thead>
            <tbody>#{rows}</tbody>
          </table>
        HTML
      end

      def file_details(by_file)
        sections = by_file.sort_by { |f, _| f }.map do |file, file_results|
          anchor = file_anchor(file)

          # Read source file
          source_lines = begin
            File.readlines(file)
          rescue
            ["(unable to read file)"]
          end

          # Build line annotations
          mutations_by_line = file_results.group_by { |r| r.mutation[:line] }

          source_html = source_lines.each_with_index.map do |line, idx|
            line_num = idx + 1
            line_mutations = mutations_by_line[line_num] || []

            line_class = if line_mutations.any? { |r| r.status == "survived" }
                           "line-survived"
                         elsif line_mutations.any? { |r| r.status == "killed" }
                           "line-killed"
                         else
                           ""
                         end

            annotation = if line_mutations.any?
                           details = line_mutations.map do |r|
                             status_icon = case r.status
                                           when "killed" then "&#x2717;"
                                           when "survived" then "&#x2713;"
                                           when "timeout" then "&#x23F1;"
                                           else "&#x26A0;"
                                           end
                             status_class = "mutation-#{r.status}"
                             "<span class=\"#{status_class}\">#{status_icon} #{h(r.mutation[:operator])}: #{h(r.mutation[:original])} &rarr; #{h(r.mutation[:replacement])}</span>"
                           end.join("<br>")
                           "<div class=\"annotations\">#{details}</div>"
                         else
                           ""
                         end

            "<tr class=\"#{line_class}\"><td class=\"line-num\">#{line_num}</td><td class=\"code\"><pre>#{h(line.chomp)}</pre>#{annotation}</td></tr>"
          end.join("\n")

          <<~HTML
            <div class="file-detail" id="#{anchor}">
              <h3>#{h(file)}</h3>
              <table class="source-table">
                #{source_html}
              </table>
            </div>
          HTML
        end.join("\n")

        "<h2>Source Files</h2>\n#{sections}"
      end

      def file_anchor(file)
        file.gsub(/[^a-zA-Z0-9]/, "-")
      end

      def h(str)
        CGI.escapeHTML(str.to_s)
      end

      def css
        <<~CSS
          * { margin: 0; padding: 0; box-sizing: border-box; }
          body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; color: #333; }
          .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
          h1 { margin-bottom: 20px; }
          h2 { margin: 30px 0 10px; }
          h3 { margin: 20px 0 8px; font-size: 1.1em; }

          .summary { display: flex; align-items: center; gap: 30px; background: #fff; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
          .score { font-size: 3em; font-weight: bold; }
          .score-high { color: #22863a; }
          .score-medium { color: #b08800; }
          .score-low { color: #cb2431; }
          .stats { display: flex; gap: 16px; flex-wrap: wrap; }
          .stat { padding: 4px 12px; border-radius: 12px; font-size: 0.9em; }
          .stat.killed { background: #dcffe4; color: #22863a; }
          .stat.survived { background: #ffdce0; color: #cb2431; }
          .stat.timeout { background: #fff5b1; color: #735c0f; }
          .stat.errored { background: #f1e5ff; color: #6f42c1; }
          .stat.total { background: #e1e4e8; color: #24292e; }

          .file-table { width: 100%; border-collapse: collapse; background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
          .file-table th, .file-table td { padding: 8px 12px; text-align: left; border-bottom: 1px solid #e1e4e8; }
          .file-table th { background: #f6f8fa; font-weight: 600; }
          .file-table td:nth-child(n+2) { text-align: right; }

          .file-detail { background: #fff; border-radius: 8px; padding: 16px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); overflow-x: auto; }
          .source-table { width: 100%; border-collapse: collapse; font-family: "SF Mono", "Fira Code", monospace; font-size: 0.85em; }
          .source-table tr { border-bottom: 1px solid #f0f0f0; }
          .line-num { width: 50px; text-align: right; padding: 2px 10px 2px 6px; color: #999; user-select: none; vertical-align: top; }
          .code { padding: 2px 8px; }
          .code pre { margin: 0; white-space: pre-wrap; word-break: break-all; }
          .line-killed { background: #f0fff4; }
          .line-survived { background: #fff5f5; }

          .annotations { margin: 2px 0 4px 20px; font-size: 0.85em; }
          .mutation-killed { color: #22863a; }
          .mutation-survived { color: #cb2431; font-weight: 600; }
          .mutation-timeout { color: #735c0f; }
          .mutation-error { color: #6f42c1; }
        CSS
      end
    end
  end
end
