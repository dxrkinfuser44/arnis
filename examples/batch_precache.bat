@echo off
REM Batch Pre-Caching Script for Arnis (Windows)
REM This script demonstrates how to pre-cache multiple regions for offline generation

setlocal enabledelayedexpansion

echo ===================================
echo Arnis Batch Pre-Caching Script
echo ===================================
echo.

REM Configuration
set "ARNIS_CMD=arnis.exe"
set "SCALE=1.0"
set "TERRAIN_FLAG=--terrain"

REM Counter variables
set /a CACHED_COUNT=0
set /a FAILED_COUNT=0
set /a TOTAL_COUNT=0

REM Define regions to cache
REM Format: name^|bbox (min_lat,min_lng,max_lat,max_lng)
set "REGION[0]=Downtown Manhattan^|40.7000,-74.0200,40.7500,-73.9700"
set "REGION[1]=Central Park^|40.7644,-73.9831,40.8006,-73.9489"
set "REGION[2]=Times Square Area^|40.7540,-73.9900,40.7600,-73.9800"

REM Count total regions
set /a TOTAL_COUNT=3

echo Starting batch pre-caching for %TOTAL_COUNT% regions...
echo.

REM Cache each region
for /L %%i in (0,1,2) do (
    REM Parse region name and bbox
    for /f "tokens=1,2 delims=|" %%a in ("!REGION[%%i]!") do (
        set "NAME=%%a"
        set "BBOX=%%b"

        echo ----------------------------------------
        echo Caching: !NAME!
        echo Bounding Box: !BBOX!
        echo ----------------------------------------

        %ARNIS_CMD% --cache-only --bbox="!BBOX!" --scale=%SCALE% %TERRAIN_FLAG%

        if !errorlevel! equ 0 (
            echo [SUCCESS] Successfully cached: !NAME!
            set /a CACHED_COUNT+=1
        ) else (
            echo [FAILED] Failed to cache: !NAME!
            set /a FAILED_COUNT+=1
        )

        echo.
        echo Waiting 5 seconds before next request...
        timeout /t 5 /nobreak >nul
        echo.
    )
)

echo ===================================
echo Batch Pre-Caching Complete!
echo ===================================
echo Successfully cached: %CACHED_COUNT% regions
echo Failed: %FAILED_COUNT% regions
echo.

REM List all cached regions
echo Listing all cached regions:
echo.
%ARNIS_CMD% --list-caches

echo.
echo To generate a world from cache, use:
echo   arnis --from-cache ^<cache_id^> --path=C:\Users\YourName\.minecraft\saves\WorldName
echo.

pause
