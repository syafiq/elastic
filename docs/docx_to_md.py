import os
import io
import re
import sys
import mammoth
from bs4 import BeautifulSoup
import markdown
import argparse

def convert_docx_to_markdown(docx_path, output_path=None):
    """
    Convert a DOCX file to Markdown format
    
    Args:
        docx_path (str): Path to the DOCX file
        output_path (str, optional): Path for the output Markdown file. If None, will use the same name as the DOCX with .md extension
    
    Returns:
        str: Path to the output Markdown file
    """
    print(f"Converting {docx_path} to Markdown...")
    
    # Set output path if not provided
    if output_path is None:
        base_name = os.path.splitext(docx_path)[0]
        output_path = f"{base_name}.md"
    
    # Convert DOCX to HTML using mammoth
    with open(docx_path, "rb") as docx_file:
        result = mammoth.convert_to_html(docx_file)
        html = result.value
    
    # Parse HTML with BeautifulSoup
    soup = BeautifulSoup(html, 'html.parser')
    
    # Process tables to make them Markdown friendly
    for table in soup.find_all('table'):
        process_table_for_markdown(table)
    
    # Process headings to make them Markdown friendly
    process_headings(soup)
    
    # Process lists
    process_lists(soup)
    
    # Process images (we can only reference them, not extract them)
    for img in soup.find_all('img'):
        alt_text = img.get('alt', 'Image')
        img.replace_with(f"![{alt_text}](image_reference)")
    
    # Convert the processed HTML to Markdown
    html_processed = str(soup)
    markdown_content = html_to_markdown(html_processed)
    
    # Add a README style header
    title = extract_title(soup) or os.path.splitext(os.path.basename(docx_path))[0]
    readme_header = f"# {title}\n\n"
    markdown_content = readme_header + markdown_content
    
    # Fix common Markdown issues
    markdown_content = fix_markdown_issues(markdown_content)
    
    # Write to output file
    with open(output_path, "w", encoding="utf-8") as output_file:
        output_file.write(markdown_content)
    
    print(f"Conversion complete. Output saved to {output_path}")
    return output_path

def process_table_for_markdown(table):
    """Process an HTML table to make it Markdown-friendly"""
    # Add a class to identify it as a table for markdown conversion
    table['class'] = 'markdown-table'
    
    rows = table.find_all('tr')
    
    # Check if the first row contains th elements
    if rows and rows[0].find('th'):
        # This is a header row
        pass
    elif rows:
        # Convert the first row to header for Markdown table
        first_row = rows[0]
        for td in first_row.find_all('td'):
            td.name = 'th'

def process_headings(soup):
    """Process headings to ensure proper Markdown structure"""
    # Find the highest level heading
    heading_levels = [int(h.name[1]) for h in soup.find_all(re.compile(r'h[1-6]'))]
    min_level = min(heading_levels) if heading_levels else 1
    
    # Normalize heading levels to start from h1
    for level in range(1, 7):
        for heading in soup.find_all(f'h{level}'):
            # Calculate new level (1-based)
            new_level = level - min_level + 1
            # Ensure it's between 1-6
            new_level = max(1, min(6, new_level))
            heading.name = f'h{new_level}'

def process_lists(soup):
    """Process lists to ensure proper Markdown structure"""
    # Process ordered lists
    for ol in soup.find_all('ol'):
        for i, li in enumerate(ol.find_all('li', recursive=False)):
            # Add numbering for explicit conversion to Markdown
            li.insert(0, f"{i+1}. ")
    
    # Process unordered lists
    for ul in soup.find_all('ul'):
        for li in ul.find_all('li', recursive=False):
            # Add bullet for explicit conversion to Markdown
            li.insert(0, "- ")

def extract_title(soup):
    """Extract the document title from the HTML"""
    # Try to find the first heading
    first_heading = soup.find(['h1', 'h2', 'h3', 'h4', 'h5', 'h6'])
    if first_heading:
        return first_heading.get_text().strip()
    return None

def html_to_markdown(html):
    """
    Convert HTML to Markdown using a custom approach
    This handles tables better than many automatic converters
    """
    soup = BeautifulSoup(html, 'html.parser')
    markdown_lines = []
    
    for element in soup.children:
        if element.name:
            process_element(element, markdown_lines)
    
    return "\n".join(markdown_lines)

def process_element(element, markdown_lines, level=0):
    """Process an HTML element and append its Markdown representation to markdown_lines"""
    if element.name is None:
        # This is a text node
        text = element.string
        if text and text.strip():
            markdown_lines.append(text.strip())
        return
    
    if element.name in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
        level_num = int(element.name[1])
        markdown_lines.append(f"{'#' * level_num} {element.get_text().strip()}")
        
    elif element.name == 'p':
        text = element.get_text().strip()
        if text:
            markdown_lines.append(text)
            markdown_lines.append("")  # Empty line after paragraph
            
    elif element.name == 'a':
        href = element.get('href', '')
        text = element.get_text().strip()
        markdown_lines.append(f"[{text}]({href})")
        
    elif element.name == 'strong' or element.name == 'b':
        markdown_lines.append(f"**{element.get_text().strip()}**")
        
    elif element.name == 'em' or element.name == 'i':
        markdown_lines.append(f"*{element.get_text().strip()}*")
        
    elif element.name == 'table' and 'markdown-table' in element.get('class', []):
        process_table(element, markdown_lines)
        markdown_lines.append("")  # Empty line after table
        
    elif element.name == 'ul':
        for li in element.find_all('li', recursive=False):
            markdown_lines.append(f"- {li.get_text().strip()}")
        markdown_lines.append("")  # Empty line after list
        
    elif element.name == 'ol':
        for i, li in enumerate(element.find_all('li', recursive=False)):
            markdown_lines.append(f"{i+1}. {li.get_text().strip()}")
        markdown_lines.append("")  # Empty line after list
        
    elif element.name in ['div', 'span', 'section', 'article']:
        # Process children for container elements
        for child in element.children:
            process_element(child, markdown_lines, level)
            
    else:
        # For other elements, process their children
        for child in element.children:
            process_element(child, markdown_lines, level)

def process_table(table, markdown_lines):
    """Process an HTML table into Markdown format"""
    rows = table.find_all('tr')
    if not rows:
        return
    
    # Process header row
    header_row = rows[0]
    headers = [cell.get_text().strip() for cell in header_row.find_all(['th', 'td'])]
    if not headers:
        return
    
    # Calculate column widths based on content
    col_widths = [max(3, len(h)) for h in headers]
    
    # Process data rows to adjust column widths
    for row in rows[1:]:
        cells = [cell.get_text().strip() for cell in row.find_all('td')]
        for i, cell in enumerate(cells):
            if i < len(col_widths):
                col_widths[i] = max(col_widths[i], len(cell))
    
    # Create header line
    header_line = "| " + " | ".join(h.ljust(w) for h, w in zip(headers, col_widths)) + " |"
    markdown_lines.append(header_line)
    
    # Create separator line
    separator_line = "| " + " | ".join("-" * w for w in col_widths) + " |"
    markdown_lines.append(separator_line)
    
    # Create data rows
    for row in rows[1:]:
        cells = [cell.get_text().strip() for cell in row.find_all('td')]
        # Ensure we have a cell for each column
        cells.extend([''] * (len(headers) - len(cells)))
        row_line = "| " + " | ".join(c.ljust(w) for c, w in zip(cells, col_widths)) + " |"
        markdown_lines.append(row_line)

def fix_markdown_issues(markdown_content):
    """Fix common Markdown issues"""
    # Fix multiple consecutive blank lines
    markdown_content = re.sub(r'\n{3,}', '\n\n', markdown_content)
    
    # Fix table formatting issues
    lines = markdown_content.split('\n')
    for i in range(len(lines) - 1):
        if lines[i].startswith('|') and not lines[i+1].startswith('|'):
            # Add a blank line after tables
            lines[i] = lines[i] + '\n'
    
    # Rejoin lines
    markdown_content = '\n'.join(lines)
    
    # Add README badges and sections if appropriate
    if '# ' in markdown_content[:100]:  # If there's a title at the start
        title_end = markdown_content.find('\n', markdown_content.find('# '))
        if title_end > 0:
            # Add badges after the title
            badges = "\n\n[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)\n"
            markdown_content = markdown_content[:title_end] + badges + markdown_content[title_end:]
    
    return markdown_content

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Convert DOCX to Markdown README')
    parser.add_argument('docx_path', help='Path to the DOCX file')
    parser.add_argument('--output', '-o', help='Path for the output Markdown file')
    
    args = parser.parse_args()
    convert_docx_to_markdown(args.docx_path, args.output)
