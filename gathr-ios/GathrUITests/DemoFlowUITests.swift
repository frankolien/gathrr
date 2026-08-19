import XCTest

final class DemoFlowUITests: XCTestCase {
    override func setUp() {
        continueAfterFailure = false
    }

    func testAHostSignsInOnboardsAndPublishesAnEventThatAppearsOnHome() {
        let app = XCUIApplication()
        app.launchArguments = ["-gathr-signed-out"]
        app.launch()

        finishOnboarding(app)
        signIn(app)
        setUpProfile(app, named: "Amara Chukwu")
        answerTheNotificationPrimer(app)

        XCTAssertTrue(
            app.staticTexts["Amara Chukwu"].waitForExistence(timeout: 25),
            "the name saved in the profile step should greet the host on Home"
        )
        attachScreenshot(app, named: "01-home-empty")

        let title = "Amara's 26th Birthday"
        publishEvent(app, titled: title)

        XCTAssertTrue(
            app.staticTexts["EVENT STARTS IN"].waitForExistence(timeout: 20),
            "publishing should land on the event with its countdown"
        )
        attachScreenshot(app, named: "03-event-detail")

        app.navigationBars.buttons.element(boundBy: 0).tap()

        XCTAssertTrue(
            app.staticTexts[title].waitForExistence(timeout: 20),
            "home must show the new event without a manual pull to refresh"
        )
        attachScreenshot(app, named: "04-home-with-event")

        openNotifications(app, expecting: title)
    }

    private func openNotifications(_ app: XCUIApplication, expecting title: String) {
        app.buttons["Notifications"].firstMatch.tap()

        XCTAssertTrue(
            app.staticTexts["Your event is live"].waitForExistence(timeout: 20),
            "publishing should leave a line in the host's own feed"
        )
        XCTAssertTrue(
            app.staticTexts[title].firstMatch.exists,
            "every notification names the event it belongs to"
        )
        attachScreenshot(app, named: "05-notifications")
    }

    private func signIn(_ app: XCUIApplication) {
        XCTAssertTrue(
            app.buttons["Continue with Apple"].waitForExistence(timeout: 30),
            "the sign-up screen should lead with the providers"
        )
        attachScreenshot(app, named: "00b-sign-up")
        signInWithAnEmailedCode(app)
    }

    @discardableResult
    private func signInWithAnEmailedCode(_ app: XCUIApplication) -> String {
        app.buttons["Continue with Email"].firstMatch.tap()

        let address = app.textFields["Email Address"]
        XCTAssertTrue(address.waitForExistence(timeout: 15), "the email step should appear")
        let mailbox = "amara.\(Int(Date().timeIntervalSince1970))@example.com"
        type(mailbox, into: address)
        app.buttons["Next"].firstMatch.tap()

        let note = app.staticTexts.matching(
            NSPredicate(format: "label BEGINSWITH %@", "Development build")
        ).firstMatch
        XCTAssertTrue(note.waitForExistence(timeout: 20), "a development build should reveal the code")
        type(String(note.label.suffix(6)), into: field("verification.code", in: app))
        app.buttons["Next"].firstMatch.tap()
        return mailbox
    }

    private func finishOnboarding(_ app: XCUIApplication) {
        let getStarted = app.buttons["Get Started"].firstMatch
        XCTAssertTrue(
            getStarted.waitForExistence(timeout: 30),
            "onboarding should be one screen ending in Get Started"
        )
        attachScreenshot(app, named: "00-onboarding")
        getStarted.tap()
    }

    private func publishEvent(_ app: XCUIApplication, titled title: String) {
        let starters = ["New event", "New Event"]
        var tapped = false
        for label in starters where !tapped {
            let button = app.buttons[label].firstMatch
            if button.waitForExistence(timeout: 5), button.isHittable {
                button.tap()
                tapped = true
            }
        }
        XCTAssertTrue(tapped, "there should be a way to start a new event from home")

        let titleField = app.textFields["What's the occasion?"].firstMatch
        type(title, into: titleField)
        attachScreenshot(app, named: "02-create-event")

        app.buttons["Publish"].firstMatch.tap()
    }

    private func setUpProfile(_ app: XCUIApplication, named name: String) {
        let heading = app.staticTexts["Your Profile"]
        XCTAssertTrue(
            heading.waitForExistence(timeout: 25),
            "verifying a code should lead to the profile step, not straight to Home"
        )

        type(name, into: field("profile.name", in: app))
        type("Lover of themed parties", into: field("profile.bio", in: app))
        attachScreenshot(app, named: "00e-profile")

        app.buttons["Save Profile"].firstMatch.tap()
    }

    private func answerTheNotificationPrimer(_ app: XCUIApplication) {
        let heading = app.staticTexts["Get Notified"]
        XCTAssertTrue(heading.waitForExistence(timeout: 20), "the notification primer should follow the profile")
        attachScreenshot(app, named: "00f-notifications")
        app.buttons["Not now"].firstMatch.tap()
    }

    private func field(_ identifier: String, in app: XCUIApplication) -> XCUIElement {
        let single = app.textFields[identifier]
        return single.waitForExistence(timeout: 5) ? single : app.textViews[identifier]
    }

    private func type(
        _ text: String,
        into field: XCUIElement,
        file: StaticString = #filePath,
        line: UInt = #line
    ) {
        XCTAssertTrue(field.waitForExistence(timeout: 15), "the field should appear", file: file, line: line)

        for attempt in 1...3 {
            field.tap()
            XCTAssertTrue(
                XCUIApplication().keyboards.element.waitForExistence(timeout: 10),
                "the keyboard should come up before typing",
                file: file,
                line: line
            )
            field.typeText(text)

            if (field.value as? String) == text { return }

            let typed = (field.value as? String) ?? ""
            if attempt < 3 {
                field.press(forDuration: 1.0)
                for _ in 0..<typed.count { field.typeText(XCUIKeyboardKey.delete.rawValue) }
            } else {
                XCTFail(
                    "typing dropped characters: wanted \(text), field holds \(typed)",
                    file: file,
                    line: line
                )
            }
        }
    }

    private func attachScreenshot(_ app: XCUIApplication, named name: String) {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
