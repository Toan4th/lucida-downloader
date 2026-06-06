#!/usr/bin/env node

const puppeteer = require('puppeteer-extra');
const StealthPlugin = require('puppeteer-extra-plugin-stealth');
puppeteer.use(StealthPlugin());

const URL = process.argv[2] || 'https://lucida.to/';

async function fetchCfClearance() {
    console.error(`Fetching cf_clearance from ${URL}...`);
    console.error('Note: This will open a browser window. Please solve any CAPTCHA if prompted.');
    console.error('Waiting for Cloudflare challenge to complete...\n');
    
    let browser;
    try {
        // Try headful mode to avoid detection
        browser = await puppeteer.launch({
            headless: false,
            executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
            ],
            defaultViewport: { width: 1400, height: 900 },
            slowMo: 100,
        });
        
        const pages = await browser.pages();
        const page = pages[0] || await browser.newPage();
        
        // Set custom user agent
        await page.setUserAgent('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36');
        
        await page.goto(URL, { waitUntil: 'networkidle2' });
        
        console.error('Page loaded. Waiting for you to solve the challenge...');
        
        // Wait for user to solve challenge - poll for cf_clearance cookie
        let attempts = 0;
        const maxAttempts = 120; // 2 minutes
        
        while (attempts < maxAttempts) {
            const client = await page.target().createCDPSession();
            const cookies = await client.send('Network.getAllCookies');
            
            const cfCookie = cookies.cookies.find(c => c.name === 'cf_clearance');
            if (cfCookie) {
                console.error('\nFound cf_clearance cookie!');
                console.log(cfCookie.value);
                await browser.close();
                process.exit(0);
            }
            
            // Check if page is still showing challenge
            const bodyText = await page.evaluate(() => document.body.innerText);
            if (!bodyText.includes('Just a moment') && 
                !bodyText.includes('Checking your browser') &&
                !bodyText.includes('Cloudflare')) {
                console.error('Challenge appears complete, waiting for cookie...');
            }
            
            await new Promise(r => setTimeout(r, 1000));
            attempts++;
            
            if (attempts % 10 === 0) {
                console.error(`Still waiting... (${attempts}s)`);
            }
        }
        
        await browser.close();
        console.error('ERROR: Timeout waiting for cf_clearance cookie');
        process.exit(1);
        
    } catch (error) {
        if (browser) await browser.close();
        console.error('ERROR:', error.message);
        process.exit(1);
    }
}

fetchCfClearance();
