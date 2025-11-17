//! Integration tests for fob-browser-test
//!
//! These tests require Chrome/Chromium to be installed and are marked #[ignore]
//! by default. Run with: cargo test --package fob-browser-test -- --ignored

use fob_browser_test::{TestBrowser, TestBrowserConfig, WaitConfig};
use std::time::Duration;

/// Helper to create a simple HTML page for testing
fn test_html_page() -> String {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
    </head>
    <body>
        <h1 id="heading">Test Heading</h1>
        <button id="test-button">Click Me</button>
        <script>
            console.log("Page loaded");
            console.warn("This is a warning");

            document.getElementById('test-button').addEventListener('click', () => {
                console.log("Button clicked");
            });
        </script>
    </body>
    </html>
    "#
    .to_string()
}

#[tokio::test]
#[ignore] // Requires Chrome to be installed
async fn test_browser_launch_and_close() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch browser");

    assert!(!browser.is_closed().await, "Browser should not be closed");

    browser
        .close()
        .await
        .expect("failed to close browser gracefully");
}

#[tokio::test]
#[ignore]
async fn test_browser_create_page() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser
        .new_page()
        .await
        .expect("failed to create page");

    // Verify we can navigate to about:blank
    page.navigate("about:blank")
        .await
        .expect("failed to navigate");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_page_navigation_data_url() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url)
        .await
        .expect("failed to navigate to data URL");

    let title = page.title().await.expect("failed to get title");
    assert_eq!(title, "Test Page", "Page title should match");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_console_capture() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Wait a bit for console messages to be captured
    tokio::time::sleep(Duration::from_millis(500)).await;

    let console = page.console();

    // Should have captured console messages
    assert!(console.len() > 0, "Should have captured console messages");

    // Should have at least one log message
    let logs = console.messages_with_level(fob_browser_test::ConsoleLevel::Log);
    assert!(
        logs.iter().any(|m| m.text.contains("Page loaded")),
        "Should have 'Page loaded' message"
    );

    // Should have a warning
    let warnings = console.warnings();
    assert!(
        warnings.iter().any(|m| m.text.contains("warning")),
        "Should have warning message"
    );

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_wait_for_selector() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Wait for an element that exists
    page.wait_for_selector("#heading", WaitConfig::default())
        .await
        .expect("failed to find #heading");

    // Wait for a button that exists
    page.wait_for_selector("#test-button", WaitConfig::default())
        .await
        .expect("failed to find #test-button");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_wait_for_selector_timeout() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Try to wait for an element that doesn't exist with short timeout
    let config = WaitConfig::new(Duration::from_millis(500), Duration::from_millis(50));

    let result = page.wait_for_selector("#non-existent", config).await;

    assert!(
        result.is_err(),
        "Should timeout waiting for non-existent element"
    );

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_wait_for_selector_injection_safety() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Try various injection attempts - all should fail to find elements
    let malicious_selectors = vec![
        r#"'); console.error('injected'); ('"#,
        r#"` + console.error('injected') + `"#,
        "#heading\n'); console.error('injected",
    ];

    for selector in malicious_selectors {
        let config = WaitConfig::new(Duration::from_millis(200), Duration::from_millis(50));
        let _ = page.wait_for_selector(selector, config).await;
        // Just checking it doesn't crash or execute injected code
    }

    // Verify no error was logged from injection
    tokio::time::sleep(Duration::from_millis(100)).await;
    let console = page.console();
    assert_eq!(
        console.error_count(),
        0,
        "Should not have executed injected code"
    );

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_evaluate_script() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Test evaluating JavaScript
    let result: String = page
        .evaluate("document.title")
        .await
        .expect("failed to evaluate");
    assert_eq!(result, "Test Page");

    // Test evaluating numbers
    let result: i32 = page.evaluate("2 + 2").await.expect("failed to evaluate");
    assert_eq!(result, 4);

    // Test evaluating booleans
    let result: bool = page
        .evaluate("!!document.getElementById('heading')")
        .await
        .expect("failed to evaluate");
    assert_eq!(result, true);

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_page_url() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    page.navigate("about:blank").await.expect("failed to navigate");

    let url = page.url().await.expect("failed to get URL");
    assert_eq!(url, "about:blank");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_screenshot() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    let screenshot = page.screenshot().await.expect("failed to take screenshot");

    // Verify we got PNG data
    assert!(!screenshot.is_empty(), "Screenshot should not be empty");
    // PNG files start with magic bytes: 89 50 4E 47
    assert_eq!(
        &screenshot[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "Screenshot should be PNG format"
    );

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_console_clear() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    let html = test_html_page();
    let data_url = format!("data:text/html,{}", urlencoding::encode(&html));

    page.navigate(&data_url).await.expect("failed to navigate");

    // Wait for console messages
    tokio::time::sleep(Duration::from_millis(500)).await;

    let console = page.console();
    assert!(console.len() > 0, "Should have messages");

    // Clear and verify
    console.clear();
    assert_eq!(console.len(), 0, "Should be empty after clear");
    assert!(console.is_empty(), "is_empty should return true");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_multiple_pages() {
    let browser = TestBrowser::launch(TestBrowserConfig::default())
        .await
        .expect("failed to launch");

    // Create multiple pages
    let page1 = browser.new_page().await.expect("failed to create page1");
    let page2 = browser.new_page().await.expect("failed to create page2");

    page1
        .navigate("about:blank")
        .await
        .expect("failed to navigate page1");
    page2
        .navigate("about:blank")
        .await
        .expect("failed to navigate page2");

    // Each page should have independent state
    let title1: String = page1
        .evaluate("document.title = 'Page 1'; document.title")
        .await
        .expect("failed to set title1");
    let title2: String = page2
        .evaluate("document.title = 'Page 2'; document.title")
        .await
        .expect("failed to set title2");

    assert_eq!(title1, "Page 1");
    assert_eq!(title2, "Page 2");

    browser.close().await.expect("failed to close");
}

#[tokio::test]
#[ignore]
async fn test_browser_config_custom_window_size() {
    let config = TestBrowserConfig::new().with_window_size(800, 600);

    let browser = TestBrowser::launch(config)
        .await
        .expect("failed to launch");

    let page = browser.new_page().await.expect("failed to create page");

    page.navigate("about:blank").await.expect("failed to navigate");

    // Check window dimensions
    let width: i32 = page
        .evaluate("window.innerWidth")
        .await
        .expect("failed to get width");
    let height: i32 = page
        .evaluate("window.innerHeight")
        .await
        .expect("failed to get height");

    // Window size might not be exact due to browser chrome, but should be close
    assert!(
        width >= 700 && width <= 900,
        "Width should be approximately 800, got {}",
        width
    );
    assert!(
        height >= 500 && height <= 700,
        "Height should be approximately 600, got {}",
        height
    );

    browser.close().await.expect("failed to close");
}
